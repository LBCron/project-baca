//! Common test utilities, test harness, and app factories for Project Baca.

use axum::body::Body;
use axum::http::{Request, Response};
use axum::Router;
use infra::{build_embedding_provider, init_db_pool, init_redis_client, AiConfig, AppConfig};
use sea_orm::DatabaseConnection;
use server::{create_app, AppState};
use std::sync::Arc;
use tower::ServiceExt;

pub struct TestHarness {
    pub app: Router,
    #[allow(dead_code)]
    pub state: Arc<AppState>,
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
        // Tests simulate edge-provided IP headers, so proxy trust is on here
        // (production keeps the default off unless the edge overwrites headers).
        let mut app_config = AppConfig::from_env().unwrap_or_default();
        app_config.server.trust_proxy_headers = true;
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

        let redis_conn = redis.get_multiplexed_tokio_connection().await.ok();

        // Use Mock provider for tests: deterministic 768-dim zero-vectors without network calls.
        let embedding = build_embedding_provider(&AiConfig {
            provider: "mock".to_string(),
            api_key: String::new(),
            model_name: "text-embedding-004".to_string(),
            dimension: 768,
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
        });

        let app = create_app(state.clone());

        Self { app, state }
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
    pub async fn live_only(&self) -> Option<redis::aio::MultiplexedConnection> {
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
