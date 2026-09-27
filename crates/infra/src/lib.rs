//! Infrastructure configuration, connection pools, and database integrations for Project Baca.

pub mod ai;
pub mod config;
pub mod email;
pub mod entities;
pub mod pool;
pub mod repositories;
pub mod security;


pub use ai::{build_embedding_provider, EmbeddingProvider};
pub use config::{
    AiConfig, AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, ServerConfig,
    StorageConfig,
};
pub use email::send_otp_email;
pub use pool::{init_db_pool, init_redis_client};
pub use repositories::{
    badge_repository::{list_badges, list_user_badges, seed_default_badges_if_empty},
    book_repository::{
        get_book_by_id, get_chapter_book_id, get_chapter_by_number, get_offline_bundle,
        list_books, search_books,
    },
    progress_repository::{get_active_progress, record_heartbeat, update_progress},
    quote_repository::{
        get_chapter_recap, get_saved_quote_by_id, get_tldr_cache, list_saved_quotes, save_quote,
        search_quotes_by_embedding, QuoteSearchRow, SavedQuoteDto,
    },
    user_repository::{
        activate_user_by_email, create_inactive_user, delete_user_by_id, find_user_by_email,
        find_user_by_id, update_inactive_credentials, update_user_password, update_user_profile,
    },
};
pub use security::{
    blacklist_access_token, generate_access_token, generate_and_store_otp, generate_refresh_token,
    hash_otp, hash_password, hash_password_async, invalidate_user_tokens, is_token_blacklisted,
    is_user_token_revoked, revoke_all_user_sessions, revoke_other_user_sessions,
    revoke_refresh_token, store_refresh_token, validate_and_rotate_refresh_token,
    verify_access_token, verify_and_consume_otp, verify_password, verify_password_async, Claims,
    DUMMY_ARGON2_HASH,
};
