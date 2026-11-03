//! Common test utilities, test harness, and app factories for Project Baca.

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::Router;
use infra::entities::{books, tags, users};
use infra::{build_embedding_provider, init_db_pool, init_redis_client, AiConfig, AppConfig};
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use server::{create_app, AppState};
use shared::{SignupRequest, VerifyOtpRequest};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle, JoinSet};
use tower::ServiceExt;
use uuid::Uuid;

/// Header the harness trusts for the client address. Tests set it to pick a
/// rate-limit bucket or lockout key the way an edge proxy would.
pub const CLIENT_IP_HEADER: &str = "cf-connecting-ip";

/// A random IPv4 address under `prefix` (two octets), drawn from 62,500 so
/// tests running in the same minute do not share a rate-limit bucket. Each
/// suite keeps its own prefix: auth 198.18, semantic 198.19, load stress
/// 198.20, reliability 198.21, catalog 198.22.
#[allow(dead_code)]
pub fn unique_ip(prefix: &str) -> String {
    let n = Uuid::new_v4().as_u128();
    format!("{prefix}.{}.{}", (n % 250) as u8, ((n >> 8) % 250) as u8)
}

pub struct TestHarness {
    pub app: Router,
    #[allow(dead_code)]
    pub state: Arc<AppState>,
    seeded: Mutex<Seeded>,
}

/// Rows a test created in the shared dev database. Tests register what they
/// seed and call `cleanup` at the end, so the catalog never fills with
/// debris; a janitor (`make purge-test-debris`) catches survivors of a
/// panicked run by their test-only markers.
#[derive(Default)]
struct Seeded {
    users: Vec<Uuid>,
    emails: Vec<String>,
    books: Vec<Uuid>,
    tags: Vec<Uuid>,
}

impl TestHarness {
    /// Initializes test harness with active database/redis connections or graceful fallbacks
    pub async fn new() -> Self {
        Self::with_config(|_| {}).await
    }

    /// Harness whose config is adjusted before the app is built, for tests
    /// that exercise optional features (public rate cap, SMTP outage).
    #[allow(dead_code)]
    pub async fn with_config(adjust: impl FnOnce(&mut AppConfig)) -> Self {
        // Tests simulate the edge-provided IP header, so proxy trust is on here
        // (production keeps the default off unless the edge overwrites headers).
        let mut app_config = AppConfig::from_env().unwrap_or_default();
        app_config.server.trust_proxy_headers = true;
        app_config.server.trusted_ip_header = CLIENT_IP_HEADER.to_string();
        adjust(&mut app_config);
        let config = Arc::new(app_config);

        let db = match init_db_pool(&config).await {
            Ok(pool) => pool,
            Err(_) => DatabaseConnection::Disconnected,
        };

        let redis = match init_redis_client(&config) {
            Ok(client) => client,
            Err(_) => redis::Client::open("redis://127.0.0.1:6380").unwrap(),
        };

        let redis_conn = infra::init_redis_conn(&redis).await.ok();

        // Use Mock provider for tests: deterministic 768-dim zero-vectors without network calls.
        let embedding = build_embedding_provider(&AiConfig {
            provider: "mock".to_string(),
            api_key: String::new(),
            model_name: "text-embedding-004".to_string(),
            dimension: 768,
            daily_budget: 0,
        })
        .expect("mock provider must always succeed");

        let storage = infra::StorageService::init(&config.storage)
            .await
            .ok()
            .map(Arc::new);

        let state = Arc::new(AppState {
            db,
            redis,
            redis_conn,
            config: Arc::clone(&config),
            embedding,
            storage,
            argon2: infra::Argon2Gate::new(config.server.argon2_max_concurrency),
        });

        let app = create_app(state.clone());

        Self {
            app,
            state,
            seeded: Mutex::new(Seeded::default()),
        }
    }

    #[allow(dead_code)]
    pub fn track_user(&self, id: Uuid) -> Uuid {
        self.seeded.lock().unwrap().users.push(id);
        id
    }

    #[allow(dead_code)]
    pub fn track_email(&self, email: &str) {
        self.seeded.lock().unwrap().emails.push(email.to_string());
    }

    #[allow(dead_code)]
    pub fn track_email_owned(&self, email: String) -> String {
        self.track_email(&email);
        email
    }

    #[allow(dead_code)]
    pub fn track_book(&self, id: Uuid) -> Uuid {
        self.seeded.lock().unwrap().books.push(id);
        id
    }

    #[allow(dead_code)]
    pub fn track_tag(&self, id: Uuid) -> Uuid {
        self.seeded.lock().unwrap().tags.push(id);
        id
    }

    /// Deletes every tracked row. Books go first so chapters, chunks,
    /// progress, quotes, and cache rows cascade before their readers do.
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        let seeded = std::mem::take(&mut *self.seeded.lock().unwrap());
        if matches!(self.state.db, DatabaseConnection::Disconnected) {
            return;
        }
        let db = &self.state.db;
        if !seeded.books.is_empty() {
            let _ = books::Entity::delete_many()
                .filter(books::Column::Id.is_in(seeded.books))
                .exec(db)
                .await;
        }
        if !seeded.tags.is_empty() {
            let _ = tags::Entity::delete_many()
                .filter(tags::Column::Id.is_in(seeded.tags))
                .exec(db)
                .await;
        }
        if !seeded.users.is_empty() {
            let _ = users::Entity::delete_many()
                .filter(users::Column::Id.is_in(seeded.users))
                .exec(db)
                .await;
        }
        if !seeded.emails.is_empty() {
            let _ = users::Entity::delete_many()
                .filter(users::Column::Email.is_in(seeded.emails))
                .exec(db)
                .await;
        }
    }

    /// Sends an HTTP request to the in-memory Axum router
    pub async fn send_request(&self, req: Request<Body>) -> Response<Body> {
        self.app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to execute in-memory request")
    }

    /// Sends an HTTP request and deserializes the response body to JSON
    ///
    /// Live-services gate: `None` when Postgres or Redis is unreachable,
    /// so live tests can skip loudly instead of passing silently.
    /// (Built per test-binary; allow dead code for binaries not using it.)
    #[allow(dead_code)]
    pub async fn live_only(&self) -> Option<infra::RedisConn> {
        if matches!(self.state.db, DatabaseConnection::Disconnected) {
            return None;
        }
        self.state.get_redis_conn().await.ok()
    }

    /// Sends an HTTP request and deserializes the response body to JSON
    #[allow(dead_code)]
    pub async fn send_json_request(
        &self,
        req: Request<Body>,
    ) -> (Response<Body>, serde_json::Value) {
        let resp = self.send_request(req).await;
        let (parts, body) = resp.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .expect("Failed to read response bytes");
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(
            |_| serde_json::json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }),
        );
        (Response::from_parts(parts, Body::from(bytes)), json)
    }
}

/// A book row in `status`, so its cover has something to answer to.
#[allow(dead_code)]
pub async fn seed_book(harness: &TestHarness, status: &str) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    use sea_orm::{ActiveModelTrait, Set};
    books::ActiveModel {
        id: Set(id),
        title: Set("Cover Probe".to_string()),
        author: Set("Probe".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Probe".to_string()),
        sub_theme: Set(None),
        description: Set("Cover probe".to_string()),
        cover_url: Set(format!("covers/{id}.webp")),
        epub_storage_path: Set(format!("epubs/{id}.epub")),
        total_words: Set(1),
        estimated_reading_minutes: Set(1),
        source_name: Set("Probe".to_string()),
        source_url: Set(None),
        license: Set("CC0".to_string()),
        publication_year: Set(None),
        status: Set(status.to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("seed book");
    harness.track_book(id)
}

/// Code a test swaps in for the one mailed to a registration.
#[allow(dead_code)]
pub const KNOWN_OTP: &str = "424242";

/// The registration id a signup answered with.
#[allow(dead_code)]
pub fn registration_of(signup_body: &serde_json::Value) -> Uuid {
    signup_body["data"]["registration_id"]
        .as_str()
        .and_then(|id| id.parse().ok())
        .expect("signup answers with a registration id")
}

/// Replaces the mailed code of a pending registration with `otp`, keeping
/// the password and name its registrant chose.
#[allow(dead_code)]
pub async fn plant_known_otp(
    redis_conn: &mut infra::RedisConn,
    email: &str,
    registration_id: Uuid,
    otp: &str,
) {
    let key = infra::registration_key(email, registration_id);
    let record: String = redis_conn.get(&key).await.expect("pending registration");
    let mut registration: infra::PendingRegistration =
        serde_json::from_str(&record).expect("registration record");
    registration.code_hash = infra::hash_otp(otp);
    let _: () = redis_conn
        .set_ex(&key, serde_json::to_string(&registration).unwrap(), 600)
        .await
        .unwrap();
}

/// Password of every account [`provision_verified_user`] creates.
#[allow(dead_code)]
pub const PROVISIONED_PASSWORD: &str = "Adversarial123!";

/// Builds a fully verified user via the real signup → OTP → verify flow and
/// returns (access_token, refresh_token). Panics loudly when services are down.
#[allow(dead_code)]
pub async fn provision_verified_user(
    harness: &TestHarness,
    redis_conn: &mut infra::RedisConn,
    tag: &str,
    ip: &str,
) -> (String, String) {
    let email = format!("adv_{tag}_{}@example.com", Uuid::new_v4());
    harness.track_email(&email);
    let password = PROVISIONED_PASSWORD;
    let signup_req = SignupRequest {
        display_name: format!("Adv{tag}"),
        email: email.clone(),
        password: password.to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header(CLIENT_IP_HEADER, ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let registration_id = registration_of(&body);
    plant_known_otp(redis_conn, &email, registration_id, KNOWN_OTP).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header(CLIENT_IP_HEADER, ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&VerifyOtpRequest {
                email: email.clone(),
                registration_id,
                otp: KNOWN_OTP.to_string(),
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    (
        body["data"]["access_token"].as_str().unwrap().to_string(),
        body["data"]["refresh_token"].as_str().unwrap().to_string(),
    )
}

/// TCP relay in front of a real service, so one app instance can lose its
/// dependency while the shared dev server keeps running for other tests.
/// `cut` drops every relayed connection and stops listening; `restore`
/// listens again on the same port. A tripping relay instead drops just the
/// connection whose client write contains every needle, which fails one
/// chosen command deterministically.
#[allow(dead_code)]
pub struct TcpRelay {
    port: u16,
    upstream: String,
    trip: &'static [&'static str],
    task: Option<JoinHandle<()>>,
}

#[allow(dead_code)]
impl TcpRelay {
    pub async fn start(upstream: &str) -> Self {
        Self::start_tripping(upstream, &[]).await
    }

    pub async fn start_tripping(upstream: &str, trip: &'static [&'static str]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("relay must bind an ephemeral port");
        let port = listener.local_addr().expect("bound address").port();
        let mut relay = Self {
            port,
            upstream: upstream.to_string(),
            trip,
            task: None,
        };
        relay.serve(listener);
        relay
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    fn serve(&mut self, listener: TcpListener) {
        let upstream = self.upstream.clone();
        let trip = self.trip;
        self.task = Some(tokio::spawn(async move {
            // Dropping the set on abort aborts every relayed connection.
            let mut connections = JoinSet::new();
            while let Ok((inbound, _)) = listener.accept().await {
                let upstream = upstream.clone();
                connections.spawn(async move {
                    if let Ok(outbound) = TcpStream::connect(&upstream).await {
                        relay_connection(inbound, outbound, trip).await;
                    }
                });
            }
        }));
    }

    pub async fn cut(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
            let _ = task.await;
        }
    }

    pub async fn restore(&mut self) {
        let listener = TcpListener::bind(("127.0.0.1", self.port))
            .await
            .expect("relay must rebind its port");
        self.serve(listener);
    }
}

async fn relay_connection(inbound: TcpStream, outbound: TcpStream, trip: &[&str]) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (mut client_read, mut client_write) = inbound.into_split();
    let (mut server_read, mut server_write) = outbound.into_split();
    let upstream = async {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = match client_read.read(&mut buf).await {
                Ok(0) | Err(_) => return,
                Ok(n) => n,
            };
            let chunk = &buf[..n];
            let tripped = !trip.is_empty()
                && trip
                    .iter()
                    .all(|needle| chunk.windows(needle.len()).any(|w| w == needle.as_bytes()));
            if tripped || server_write.write_all(chunk).await.is_err() {
                return;
            }
        }
    };
    let downstream = tokio::io::copy(&mut server_read, &mut client_write);
    tokio::select! {
        _ = upstream => {},
        _ = downstream => {},
    }
}

/// Redis URL of `config` rerouted through a relay on `port`.
#[allow(dead_code)]
pub fn redis_url_via(config: &AppConfig, port: u16) -> String {
    config
        .redis
        .url
        .replacen(&redis_upstream(config), &format!("127.0.0.1:{port}"), 1)
}

/// `host:port` of the Redis server a config points at.
#[allow(dead_code)]
pub fn redis_upstream(config: &AppConfig) -> String {
    let client = redis::Client::open(config.redis.url.as_str()).expect("valid REDIS_URL");
    match &client.get_connection_info().addr {
        redis::ConnectionAddr::Tcp(host, port) => format!("{host}:{port}"),
        other => panic!("relay supports plain TCP Redis only, got {other:?}"),
    }
}

/// Listener that accepts connections and never answers, the way a wedged
/// object store behaves.
#[allow(dead_code)]
pub async fn silent_listener() -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("silent listener must bind");
    let port = listener.local_addr().expect("bound address").port();
    let task = tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((socket, _)) = listener.accept().await {
            held.push(socket);
        }
    });
    (port, task)
}
