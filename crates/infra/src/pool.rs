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
