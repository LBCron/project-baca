//! Strongly-typed modular configuration for Project Baca.
//! Reads environment variables with fallback defaults matching `.env.example`.

use serde::{Deserialize, Serialize};
use shared::AppError;
use std::env;

/// Core Axum HTTP Server configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub env: String,
    pub cors_allowed_origins: Vec<String>,
    pub log_format: String,
    /// Trust `CF-Connecting-IP` / `X-Real-IP` / `X-Forwarded-For` for rate
    /// limiting. Keep false unless the edge proxy overwrites these headers
    /// (spoofable by direct clients otherwise).
    pub trust_proxy_headers: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            env: "development".to_string(),
            cors_allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ],
            log_format: "text".to_string(),
            trust_proxy_headers: false,
        }
    }
}

/// PostgreSQL 17 + pgvector connection configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://baca_user:baca_password@localhost:5433/project_baca_db".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout_secs: 10,
            idle_timeout_secs: 300,
        }
    }
}

/// Redis 7 connection configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6380/0".to_string(),
        }
    }
}

/// Authentication and JWT security configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_expiry_minutes: u64,
    pub refresh_expiry_days: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "super-secret-jwt-key-replace-with-at-least-32-random-characters"
                .to_string(),
            access_expiry_minutes: 1440, // 24 hours / 1 day
            refresh_expiry_days: 14,     // 14 days
        }
    }
}

/// S3 / MinIO object storage configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_epubs: String,
    pub bucket_covers: String,
    pub force_path_style: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9005".to_string(),
            region: "auto".to_string(),
            access_key_id: "minioadmin".to_string(),
            secret_access_key: "minioadmin".to_string(),
            bucket_epubs: "baca-epubs".to_string(),
            bucket_covers: "baca-covers".to_string(),
            force_path_style: true,
        }
    }
}

/// Transactional email configuration (Mailpit / SMTP)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 1025,
            smtp_user: String::new(),
            smtp_password: String::new(),
            smtp_from_email: "no-reply@projectbaca.local".to_string(),
            smtp_from_name: "Project Baca".to_string(),
        }
    }
}

/// AI and Embedding provider configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: String,
    pub model_name: String,
    pub dimension: usize,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "gemini".to_string(),
            api_key: String::new(),
            model_name: "gemini-embedding-2".to_string(),
            dimension: 768,
        }
    }
}

/// Master application configuration aggregating all modular sub-configurations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub email: EmailConfig,
    pub ai: AiConfig,
}

impl AppConfig {
    /// Loads config from env with `.env` support.
    pub fn from_env() -> Result<Self, AppError> {
        let _ = dotenvy::dotenv();

        let default_server = ServerConfig::default();
        let default_db = DatabaseConfig::default();
        let default_redis = RedisConfig::default();
        let default_auth = AuthConfig::default();
        let default_storage = StorageConfig::default();
        let default_email = EmailConfig::default();
        let default_ai = AiConfig::default();

        let host = env::var("APP_HOST").unwrap_or(default_server.host);
        let port = env::var("APP_PORT")
            .or_else(|_| env::var("PORT"))
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(default_server.port);
        let env_mode = env::var("APP_ENV").unwrap_or(default_server.env);
        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .map(|val| {
                val.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or(default_server.cors_allowed_origins);
        let log_format = env::var("LOG_FORMAT").unwrap_or(default_server.log_format);
        let trust_proxy_headers = env::var("TRUST_PROXY_HEADERS")
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
            .unwrap_or(default_server.trust_proxy_headers);

        let server = ServerConfig {
            host,
            port,
            env: env_mode,
            cors_allowed_origins,
            log_format,
            trust_proxy_headers,
        };

        let db_url = env::var("DATABASE_URL").unwrap_or(default_db.url);
        let db_max_conn = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_db.max_connections);
        let db_min_conn = env::var("DATABASE_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_db.min_connections);
        let db_connect_timeout = env::var("DATABASE_CONNECT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_db.connect_timeout_secs);
        let db_idle_timeout = env::var("DATABASE_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_db.idle_timeout_secs);

        let database = DatabaseConfig {
            url: db_url,
            max_connections: db_max_conn,
            min_connections: db_min_conn,
            connect_timeout_secs: db_connect_timeout,
            idle_timeout_secs: db_idle_timeout,
        };

        let redis_url = env::var("REDIS_URL").unwrap_or(default_redis.url);
        let redis = RedisConfig { url: redis_url };

        let jwt_secret = env::var("JWT_SECRET").unwrap_or(default_auth.jwt_secret.clone());
        // Fail closed: a default or short secret in production would let anyone
        // forge tokens, including admin ones.
        if server.env.eq_ignore_ascii_case("production")
            && (jwt_secret == AuthConfig::default().jwt_secret || jwt_secret.len() < 32)
        {
            panic!(
                "JWT_SECRET must be set to a unique value of at least 32 characters in production"
            );
        }
        let access_expiry = env::var("JWT_ACCESS_EXPIRY_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_auth.access_expiry_minutes);
        let refresh_expiry = env::var("JWT_REFRESH_EXPIRY_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_auth.refresh_expiry_days);

        let auth = AuthConfig {
            jwt_secret,
            access_expiry_minutes: access_expiry,
            refresh_expiry_days: refresh_expiry,
        };

        let s3_endpoint = env::var("S3_ENDPOINT")
            .or_else(|_| env::var("MINIO_ENDPOINT"))
            .unwrap_or(default_storage.endpoint);
        let s3_region = env::var("S3_REGION").unwrap_or(default_storage.region);
        let s3_access_key = env::var("S3_ACCESS_KEY_ID").unwrap_or(default_storage.access_key_id);
        let s3_secret_key =
            env::var("S3_SECRET_ACCESS_KEY").unwrap_or(default_storage.secret_access_key);
        let s3_bucket_epubs = env::var("S3_BUCKET_EPUBS")
            .or_else(|_| env::var("MINIO_BUCKET"))
            .unwrap_or(default_storage.bucket_epubs);
        let s3_bucket_covers =
            env::var("S3_BUCKET_COVERS").unwrap_or(default_storage.bucket_covers);
        let s3_force_path_style = env::var("S3_FORCE_PATH_STYLE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_storage.force_path_style);

        let storage = StorageConfig {
            endpoint: s3_endpoint,
            region: s3_region,
            access_key_id: s3_access_key,
            secret_access_key: s3_secret_key,
            bucket_epubs: s3_bucket_epubs,
            bucket_covers: s3_bucket_covers,
            force_path_style: s3_force_path_style,
        };

        let smtp_host = env::var("SMTP_HOST").unwrap_or(default_email.smtp_host);
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_email.smtp_port);
        let smtp_user = env::var("SMTP_USER").unwrap_or(default_email.smtp_user);
        let smtp_password = env::var("SMTP_PASSWORD").unwrap_or(default_email.smtp_password);
        let smtp_from_email = env::var("SMTP_FROM_EMAIL").unwrap_or(default_email.smtp_from_email);
        let smtp_from_name = env::var("SMTP_FROM_NAME").unwrap_or(default_email.smtp_from_name);

        let email = EmailConfig {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            smtp_from_email,
            smtp_from_name,
        };

        let ai_provider = env::var("EMBEDDING_PROVIDER")
            .or_else(|_| env::var("AI_PROVIDER"))
            .unwrap_or(default_ai.provider);
        let ai_api_key = env::var("GEMINI_API_KEY")
            .or_else(|_| env::var("AI_API_KEY"))
            .unwrap_or(default_ai.api_key);
        let ai_model_name = env::var("EMBEDDING_MODEL_NAME")
            .or_else(|_| env::var("AI_MODEL_NAME"))
            .unwrap_or(default_ai.model_name);
        let ai_dimension = env::var("EMBEDDING_DIMENSION")
            .or_else(|_| env::var("AI_DIMENSION"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_ai.dimension);

        let ai = AiConfig {
            provider: ai_provider,
            api_key: ai_api_key,
            model_name: ai_model_name,
            dimension: ai_dimension,
        };

        Ok(Self {
            server,
            database,
            redis,
            auth,
            storage,
            email,
            ai,
        })
    }

    /// Accessor for server port
    pub fn port(&self) -> u16 {
        self.server.port
    }

    /// Whether the server runs in production (Swagger disabled, secrets strict).
    pub fn is_production(&self) -> bool {
        self.server.env.eq_ignore_ascii_case("production")
    }

    /// Accessor for database connection URL
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Accessor for redis connection URL
    pub fn redis_url(&self) -> &str {
        &self.redis.url
    }

    /// Accessor for JWT secret key
    pub fn jwt_secret(&self) -> &str {
        &self.auth.jwt_secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_config() {
        let config = AppConfig::default();

        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(!config.server.trust_proxy_headers);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.database.min_connections, 5);
        assert_eq!(config.auth.access_expiry_minutes, 1440);
        assert_eq!(config.auth.refresh_expiry_days, 14);
        assert_eq!(config.storage.bucket_epubs, "baca-epubs");
        assert_eq!(config.ai.provider, "gemini");
        assert_eq!(config.ai.dimension, 768);
    }

    #[test]
    fn test_config_accessors() {
        let config = AppConfig::default();

        assert_eq!(config.port(), 8080);
        assert_eq!(
            config.database_url(),
            "postgres://baca_user:baca_password@localhost:5433/project_baca_db"
        );
        assert_eq!(config.redis_url(), "redis://localhost:6380/0");
        assert_eq!(
            config.jwt_secret(),
            "super-secret-jwt-key-replace-with-at-least-32-random-characters"
        );
    }

    #[test]
    fn test_app_config_from_env_clean_execution() {
        let config_res = AppConfig::from_env();
        assert!(config_res.is_ok());

        let config = config_res.unwrap();
        assert!(config.server.port > 0);
        assert!(!config.database.url.is_empty());
        assert!(!config.redis.url.is_empty());
    }
}
