//! Project Baca — HTTP REST API Server
//! Built with Axum 0.8, SeaORM, Redis, and OpenAPI (utoipa).

use infra::{build_embedding_provider, init_db_pool, init_redis_client, AppConfig};
use sea_orm::DatabaseConnection;
use server::{create_app, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "server=debug,infra=debug,tower_http=debug".into());

    if log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    info!("Starting Project Baca API Server...");

    let config = Arc::new(AppConfig::from_env().map_err(|e| {
        eprintln!("Configuration initialization failed: {e}");
        e
    })?);

    // Attempt DB and Redis initialization (graceful fallback if offline during initial build check)
    let db = match init_db_pool(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::warn!("Postgres connection failed ({e}). Running in standalone mode.");
            DatabaseConnection::Disconnected
        }
    };

    let redis = match init_redis_client(&config) {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!("Redis connection failed ({e}).");
            redis::Client::open("redis://127.0.0.1:6380")?
        }
    };

    let redis_conn = match redis.get_multiplexed_tokio_connection().await {
        Ok(conn) => {
            tracing::info!("Pre-initialized shared multiplexed Redis connection");
            Some(conn)
        }
        Err(e) => {
            tracing::warn!("Failed to pre-initialize multiplexed Redis connection: {e}");
            None
        }
    };

    // Initialize dual-mode embedding provider (Gemini REST API or FastEmbed CPU).
    // Graceful fallback: if the API key is absent, fall back to FastEmbed stub so the
    // server starts cleanly. Handlers requiring embeddings will return a descriptive error.
    let embedding = match build_embedding_provider(&config.ai) {
        Ok(provider) => {
            tracing::info!(
                provider = %config.ai.provider,
                model = %config.ai.model_name,
                dimension = config.ai.dimension,
                "Embedding provider initialized"
            );
            provider
        }
        Err(e) => {
            tracing::warn!(
                "Embedding provider initialization failed ({e}). \
                 Falling back to FastEmbed stub. \
                 Set AI_API_KEY to enable Gemini embeddings."
            );
            build_embedding_provider(&infra::AiConfig {
                provider: "fastembed".to_string(),
                api_key: String::new(),
                model_name: config.ai.model_name.clone(),
                dimension: config.ai.dimension,
            })
            .unwrap_or_else(|_| unreachable!("fastembed provider always succeeds"))
        }
    };

    // Object storage (MinIO/S3). Optional: uploads fail closed when unreachable.
    let storage = match infra::StorageService::init(&config.storage).await {
        Ok(service) => {
            tracing::info!("Object storage initialized");
            Some(Arc::new(service))
        }
        Err(e) => {
            tracing::warn!("Object storage unavailable ({e}). Admin upload disabled.");
            None
        }
    };

    let state = Arc::new(AppState {
        db,
        redis,
        redis_conn,
        config: Arc::clone(&config),
        embedding,
        storage,
    });
    let app = create_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port()));
    info!("Server listening on http://{}", addr);
    info!("OpenAPI Swagger UI available on http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to install CTRL+C signal handler: {err}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                tracing::error!("Failed to install SIGTERM signal handler: {err}");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Signal received, starting graceful shutdown...");
}
