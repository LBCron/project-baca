//! Fault injection and graceful degradation.

use crate::common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{provision_verified_user, TestHarness, PROVISIONED_PASSWORD};
use infra::{build_embedding_provider, AiConfig, AppConfig, StorageService, S3_OPERATION_TIMEOUT};
use mockall::automock;
use pretty_assertions::assert_eq;
use sea_orm::DatabaseConnection;
use server::{create_app, AppState};
use shared::{AppError, BookSummaryDto};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use uuid::Uuid;

#[automock]
pub trait BookCatalogPort: Send + Sync {
    fn find_book_by_id(&self, id: &Uuid) -> Result<Option<BookSummaryDto>, AppError>;
}

/// App with no database and no shared Redis handle, for fault injection.
fn offline_app(config: AppConfig, storage: Option<StorageService>) -> axum::Router {
    let embedding = build_embedding_provider(&AiConfig {
        provider: "fastembed".to_string(),
        api_key: String::new(),
        model_name: "text-embedding-004".to_string(),
        dimension: 768,
        daily_budget: 0,
    })
    .expect("fastembed stub must always succeed");

    create_app(Arc::new(AppState {
        db: DatabaseConnection::Disconnected,
        redis: redis::Client::open("redis://127.0.0.1:6380").unwrap(),
        redis_conn: None,
        config: Arc::new(config),
        embedding,
        storage: storage.map(Arc::new),
        argon2: infra::Argon2Gate::new(2),
    }))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn get_bearer(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn test_reliability_disconnected_database_fallback() {
    let mut config = AppConfig::default();
    config.database.url = "postgresql://invalid_host:5433/none".to_string();
    let app = offline_app(config, None);

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let (resp, body) = {
        let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
        let (parts, body) = resp.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (
            axum::http::Response::from_parts(parts, Body::from(bytes)),
            json,
        )
    };

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["data"]["postgres"], "disconnected");
}

#[tokio::test]
async fn test_reliability_mock_repository_fault_injection() {
    let mut mock_port = MockBookCatalogPort::new();
    let sample_id = Uuid::new_v4();

    mock_port
        .expect_find_book_by_id()
        .with(mockall::predicate::eq(sample_id))
        .times(1)
        .returning(|_| {
            Err(AppError::Internal(
                "Database connection pool exhausted".to_string(),
            ))
        });

    let result = mock_port.find_book_by_id(&sample_id);

    claims::assert_err!(&result);
    match result {
        Err(AppError::Internal(msg)) => {
            assert!(msg.contains("pool exhausted"));
        }
        _ => panic!("Expected AppError::Internal"),
    }
}

#[tokio::test]
async fn test_reliability_oversized_request_id_handling() {
    let harness = TestHarness::new().await;
    let long_id = "req_".to_string() + &"a".repeat(1000);

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .header("x-request-id", long_id)
        .body(Body::empty())
        .unwrap();

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let returned = resp.headers()["x-request-id"].to_str().unwrap();
    assert!(
        Uuid::parse_str(returned).is_ok(),
        "oversized client id must be replaced by a server UUID, got {returned}"
    );
    harness.cleanup().await;
}

/// Signup fails closed when the mail relay is unreachable: no 201 without a
/// deliverable code, and the OTP cooldown must not block the retry.
#[tokio::test]
async fn test_signup_fails_closed_when_smtp_unreachable() {
    let harness = TestHarness::with_config(|c| {
        c.email.smtp_host = "127.0.0.1".to_string();
        c.email.smtp_port = 1;
    })
    .await;
    if harness.live_only().await.is_none() {
        panic!("live Postgres + Redis required (db-up)");
    }
    let email = format!("smtp_down_{}@example.com", uuid::Uuid::new_v4());
    // A fresh address per run keeps the per-IP signup cap out of the picture.
    let ip = common::unique_ip("198.21");
    harness.track_email(&email);
    let signup = || {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/signup")
            .header(common::CLIENT_IP_HEADER, &ip)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "display_name": "Relay Down",
                    "email": email,
                    "password": "RelayDown123!"
                })
                .to_string(),
            ))
            .unwrap()
    };

    let (resp, body) = harness.send_json_request(signup()).await;
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "EXTERNAL_SERVICE_ERROR");

    // Immediate retry hits the relay again instead of the 60 s cooldown.
    let (resp, body) = harness.send_json_request(signup()).await;
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "EXTERNAL_SERVICE_ERROR");

    // No mail went out, so none counts against the address.
    let mut redis = harness.live_only().await.expect("live Redis");
    let sent: Option<i64> = redis::AsyncCommands::get(&mut redis, format!("otp:sent:3600:{email}"))
        .await
        .unwrap();
    assert_eq!(sent.unwrap_or(0), 0);
    harness.cleanup().await;
}

/// Redis goes away under a running API and comes back: bearer routes answer
/// 503 (so clients keep their tokens) while health reports Redis down, and
/// the same process serves login and `/me` again without a restart.
#[tokio::test]
async fn test_redis_outage_answers_503_and_heals_without_restart() {
    let base = AppConfig::from_env().unwrap_or_default();
    let mut relay = common::TcpRelay::start(&common::redis_upstream(&base)).await;
    let relay_url = common::redis_url_via(&base, relay.port());
    let harness = TestHarness::with_config(|c| c.redis.url = relay_url).await;
    let mut redis = harness
        .live_only()
        .await
        .expect("live Postgres + Redis required (db-up)");
    let ip = common::unique_ip("198.21");
    let (access, _) = provision_verified_user(&harness, &mut redis, "relay", &ip).await;
    let (resp, me) = harness
        .send_json_request(get_bearer("/api/v1/me", &access))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let email = me["data"]["email"].as_str().unwrap().to_string();

    relay.cut().await;
    let resp = harness
        .send_request(get_bearer("/api/v1/me", &access))
        .await;
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let (_, health) = harness.send_json_request(get("/api/v1/health")).await;
    assert_eq!(health["data"]["redis"], "disconnected");

    relay.restore().await;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let resp = harness
            .send_request(get_bearer("/api/v1/me", &access))
            .await;
        if resp.status() == StatusCode::OK {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "API still answers {} 5 s after Redis returned",
            resp.status()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(common::CLIENT_IP_HEADER, &ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "email": email, "password": PROVISIONED_PASSWORD }).to_string(),
        ))
        .unwrap();
    assert_eq!(harness.send_request(login).await.status(), StatusCode::OK);
    harness.cleanup().await;
}

const PRODUCTION_CORS: &str = "https://rotaria.site,tauri://localhost,http://tauri.localhost";

/// A wildcard or a malformed origin in the production allowlist stops the
/// boot before anything listens.
#[tokio::test]
async fn test_production_boot_with_a_loose_cors_origin_exits_nonzero() {
    for origins in ["*", "https://rotaria.site/,tauri://localhost"] {
        let run = tokio::process::Command::new(env!("CARGO_BIN_EXE_server"))
            .current_dir(std::env::temp_dir())
            .env_clear()
            .env("APP_ENV", "production")
            .env("JWT_SECRET", "p".repeat(48))
            .env("SMTP_SECURITY", "starttls")
            .env("TRUST_PROXY_HEADERS", "true")
            .env("TRUSTED_IP_HEADER", "CF-Connecting-IP")
            .env("OTP_HOURLY_BUDGET", "100")
            .env("AI_DAILY_BUDGET", "5000")
            .env("CORS_ALLOWED_ORIGINS", origins)
            .kill_on_drop(true)
            .output();
        let output = tokio::time::timeout(Duration::from_secs(30), run)
            .await
            .expect("server must refuse its config at once")
            .expect("server binary must start");
        assert!(!output.status.success(), "{origins}: {:?}", output.status);
        let log = String::from_utf8_lossy(&output.stderr);
        assert!(log.contains("CORS_ALLOWED_ORIGINS"), "{origins}: {log}");
    }
}

/// A production process that cannot reach Postgres exits non-zero so the
/// orchestrator restarts it, instead of serving errors forever.
#[tokio::test]
async fn test_production_boot_without_postgres_exits_nonzero() {
    let run = tokio::process::Command::new(env!("CARGO_BIN_EXE_server"))
        .current_dir(std::env::temp_dir())
        .env_clear()
        .env("APP_ENV", "production")
        .env("DATABASE_URL", "postgres://nobody:nothing@127.0.0.1:1/none")
        .env("DATABASE_CONNECT_TIMEOUT_SECS", "2")
        .env("JWT_SECRET", "p".repeat(48))
        .env("SMTP_SECURITY", "starttls")
        .env("TRUST_PROXY_HEADERS", "true")
        .env("TRUSTED_IP_HEADER", "CF-Connecting-IP")
        .env("OTP_HOURLY_BUDGET", "100")
        .env("AI_DAILY_BUDGET", "5000")
        .env("CORS_ALLOWED_ORIGINS", PRODUCTION_CORS)
        .env("RUST_LOG", "server=info")
        .kill_on_drop(true)
        .output();
    let output = tokio::time::timeout(Duration::from_secs(30), run)
        .await
        .expect("server must exit instead of serving without Postgres")
        .expect("server binary must start");
    assert!(!output.status.success(), "exit status {:?}", output.status);
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        log.contains("Postgres connection failed in production"),
        "unexpected boot log: {log}"
    );
}

/// A production process whose object store refuses its credentials exits
/// non-zero instead of booting with uploads and covers silently broken.
#[tokio::test]
async fn test_production_boot_with_wrong_storage_secret_exits_nonzero() {
    let live = AppConfig::from_env().expect("dev config");
    let run = tokio::process::Command::new(env!("CARGO_BIN_EXE_server"))
        .current_dir(std::env::temp_dir())
        .env_clear()
        .env("APP_ENV", "production")
        .env("APP_HOST", "127.0.0.1")
        .env("APP_PORT", "0")
        .env("DATABASE_URL", &live.database.url)
        .env("REDIS_URL", &live.redis.url)
        .env("S3_ENDPOINT", &live.storage.endpoint)
        .env("S3_ACCESS_KEY_ID", &live.storage.access_key_id)
        .env("S3_SECRET_ACCESS_KEY", "not-the-secret")
        .env("JWT_SECRET", "p".repeat(48))
        .env("SMTP_SECURITY", "starttls")
        .env("TRUST_PROXY_HEADERS", "true")
        .env("TRUSTED_IP_HEADER", "CF-Connecting-IP")
        .env("OTP_HOURLY_BUDGET", "100")
        .env("AI_DAILY_BUDGET", "5000")
        .env("CORS_ALLOWED_ORIGINS", PRODUCTION_CORS)
        .env("RUST_LOG", "server=info")
        .kill_on_drop(true)
        .output();
    let output = tokio::time::timeout(Duration::from_secs(30), run)
        .await
        .expect("server must exit instead of serving without storage")
        .expect("server binary must start");
    assert!(!output.status.success(), "exit status {:?}", output.status);
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        log.contains("Object storage unavailable in production") && log.contains("HTTP 403"),
        "unexpected boot log: {log}"
    );
}

/// An object store that accepts the connection and never answers fails the
/// cover request within the operation timeout instead of holding it open.
#[tokio::test]
async fn test_wedged_object_store_fails_cover_within_operation_timeout() {
    // The cover route asks the database whether the book is published before
    // it reads the bucket, so this one needs the live database.
    let harness = common::TestHarness::new().await;
    let book_id = common::seed_book(&harness, "published").await;
    let (port, _listener) = common::silent_listener().await;
    let mut config = AppConfig::default();
    config.storage.endpoint = format!("http://127.0.0.1:{port}");
    let storage = StorageService::connect(&config.storage);
    let embedding = build_embedding_provider(&AiConfig {
        provider: "fastembed".to_string(),
        api_key: String::new(),
        model_name: "text-embedding-004".to_string(),
        dimension: 768,
        daily_budget: 0,
    })
    .expect("fastembed stub must always succeed");
    let app = create_app(Arc::new(AppState {
        db: harness.state.db.clone(),
        redis: redis::Client::open("redis://127.0.0.1:6380").unwrap(),
        redis_conn: None,
        config: Arc::new(config),
        embedding,
        storage: Some(Arc::new(storage)),
        argon2: infra::Argon2Gate::new(2),
    }));

    let started = Instant::now();
    let uri = format!("/api/v1/covers/{book_id}.webp");
    let resp = tokio::time::timeout(
        S3_OPERATION_TIMEOUT + Duration::from_secs(5),
        app.oneshot(get(&uri)),
    )
    .await
    .expect("cover request must not hang on a wedged store")
    .unwrap();
    assert!(resp.status().is_server_error(), "got {}", resp.status());
    assert!(
        started.elapsed() <= S3_OPERATION_TIMEOUT + Duration::from_secs(2),
        "took {:?}",
        started.elapsed()
    );
    harness.cleanup().await;
}
