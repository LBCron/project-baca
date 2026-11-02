//! Dynamic connection pool initializers for PostgreSQL (SeaORM) and Redis.

use crate::config::AppConfig;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use shared::AppError;
use std::time::Duration;
use tracing::info;

/// `scheme://host:port` of a connection URL, with credentials, path, and
/// query dropped so the line is safe to log.
pub fn redacted_endpoint(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return "<unparseable url>".to_string();
    };
    let (scheme, rest) = url.split_at(scheme_end);
    let rest = &rest[3..];
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority.rsplit('@').next().unwrap_or_default();
    format!("{scheme}://{host_port}")
}

/// Initializes PostgreSQL connection pool with SeaORM using settings from `AppConfig`
pub async fn init_db_pool(config: &AppConfig) -> Result<DatabaseConnection, AppError> {
    info!(
        "Connecting to PostgreSQL at {} (max_conn: {}, min_conn: {})",
        redacted_endpoint(&config.database.url),
        config.database.max_connections,
        config.database.min_connections
    );

    let mut opt = ConnectOptions::new(&config.database.url);
    opt.max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect_timeout(Duration::from_secs(config.database.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_secs))
        .sqlx_logging(false);

    Database::connect(opt)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {e}")))
}

/// Initializes Redis connection client using settings from `AppConfig`
pub fn init_redis_client(config: &AppConfig) -> Result<redis::Client, AppError> {
    info!(
        "Connecting to Redis at {}",
        redacted_endpoint(&config.redis.url)
    );

    redis::Client::open(config.redis.url.as_str())
        .map_err(|e| AppError::Internal(format!("Failed to initialize Redis client: {e}")))
}

/// Shared Redis handle that reconnects after the server restarts or the
/// socket drops. Cloning is cheap; every clone shares one socket.
pub type RedisConn = redis::aio::ConnectionManager;

/// Seconds a client should wait before retrying while Redis is unreachable.
pub const REDIS_RETRY_AFTER_SECS: u64 = 5;

/// Opens the shared [`RedisConn`].
///
/// The manager's own backoff starts at one second and multiplies by its
/// factor up to a minute, which would park requests for the whole outage.
/// With no internal retries a request during an outage fails within the
/// connect timeout, and the next request after recovery triggers the
/// reconnect, so callers answer 503 quickly and heal without a restart.
pub async fn init_redis_conn(client: &redis::Client) -> Result<RedisConn, AppError> {
    let config = redis::aio::ConnectionManagerConfig::new()
        .set_number_of_retries(0)
        .set_connection_timeout(Duration::from_secs(1))
        .set_response_timeout(Duration::from_secs(3));
    redis::aio::ConnectionManager::new_with_config(client.clone(), config)
        .await
        .map_err(|e| {
            tracing::warn!(target: "infra::redis", error = %e, "Redis unreachable");
            AppError::ServiceUnavailable {
                retry_after: REDIS_RETRY_AFTER_SECS,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::redacted_endpoint;

    #[test]
    fn redacted_endpoint_drops_credentials_and_path() {
        assert_eq!(
            redacted_endpoint(
                "postgres://baca_user:s3cr%40t@db.internal:5432/project?sslmode=require"
            ),
            "postgres://db.internal:5432"
        );
        assert_eq!(
            redacted_endpoint("redis://:password@redis:6379/0"),
            "redis://redis:6379"
        );
        assert_eq!(
            redacted_endpoint("redis://localhost:6380/0"),
            "redis://localhost:6380"
        );
        assert_eq!(redacted_endpoint("garbage"), "<unparseable url>");
    }
}
