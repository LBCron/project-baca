//! HTTP REST API server.

use infra::{
    build_embedding_provider, init_db_pool, init_redis_client, init_redis_conn, AppConfig,
};
use sea_orm::DatabaseConnection;
use server::{create_app, AppState};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,tower_http=warn".into());

    if log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .with(server::error_tracking::tracing_layer())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(server::error_tracking::tracing_layer())
            .init();
    }

    info!("Starting Project Baca API Server...");

    let config = Arc::new(AppConfig::from_env().map_err(|e| {
        eprintln!("Configuration initialization failed: {e}");
        e
    })?);
    let _sentry = init_error_tracking(&config);

    // Production refuses to boot without its stores so the orchestrator
    // restarts the container; development keeps running for offline work.
    let db = match init_db_pool(&config).await {
        Ok(pool) => pool,
        Err(e) if config.is_production() => {
            tracing::error!("Postgres connection failed in production ({e}). Refusing to boot.");
            return Err(e.into());
        }
        Err(e) => {
            tracing::warn!("Postgres connection failed ({e}). Running in standalone mode.");
            DatabaseConnection::Disconnected
        }
    };

    let redis = match init_redis_client(&config) {
        Ok(client) => client,
        Err(e) => {
            if config.is_production() {
                tracing::error!("Redis connection failed in production ({e}). Refusing to boot with a silent fallback.");
                return Err(e.into());
            }
            tracing::warn!("Redis connection failed ({e}).");
            redis::Client::open("redis://127.0.0.1:6380")?
        }
    };

    let redis_conn = match init_redis_conn(&redis).await {
        Ok(conn) => {
            tracing::info!("Shared Redis connection ready");
            Some(conn)
        }
        Err(e) if config.is_production() => {
            tracing::error!("Redis unreachable in production ({e}). Refusing to boot.");
            return Err(e.into());
        }
        Err(e) => {
            tracing::warn!("Redis unreachable at boot ({e}); connecting per request.");
            None
        }
    };

    // The model service may still be loading; quote search answers 503
    // AI_UNAVAILABLE until it is, so the server boots without waiting.
    let embedding = build_embedding_provider(&config.ai)
        .map_err(|e| format!("Embedding provider failed to start: {e}"))?;

    // Object storage (R2/MinIO). Production refuses to boot without it, since
    // uploads and covers would fail silently; development keeps running.
    let storage = match infra::StorageService::init(&config.storage).await {
        Ok(service) => {
            tracing::info!("Object storage initialized");
            Some(Arc::new(service))
        }
        Err(e) if config.is_production() => {
            tracing::error!("Object storage unavailable in production ({e}). Refusing to boot.");
            return Err(e.into());
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
        argon2: infra::Argon2Gate::new(config.server.argon2_max_concurrency),
    });
    let app = create_app(state);

    let addr = config.bind_addr()?;
    info!("Server listening on http://{}", addr);
    if !config.is_production() {
        info!("OpenAPI Swagger UI available on http://{}/swagger-ui", addr);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Sentry when `SENTRY_DSN` holds a DSN; off when it is unset or empty. A
/// malformed DSN is logged and leaves tracking off rather than stopping the
/// boot. The guard flushes queued events when `main` returns.
fn init_error_tracking(config: &AppConfig) -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").unwrap_or_default();
    if dsn.trim().is_empty() {
        return None;
    }
    match dsn.trim().parse::<sentry::types::Dsn>() {
        Ok(dsn) => {
            let mut options = server::error_tracking::options(&config.server.env);
            options.dsn = Some(dsn);
            info!("Error tracking on");
            Some(sentry::init(options))
        }
        Err(e) => {
            tracing::error!("SENTRY_DSN is not a valid DSN ({e}); error tracking off");
            None
        }
    }
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
