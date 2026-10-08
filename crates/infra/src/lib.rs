//! Pools, config, entities, and repositories.

pub mod ai;
pub mod config;
pub mod email;
pub mod entities;
pub mod pool;
pub mod queue;
pub mod repositories;
pub mod security;
pub mod storage;

pub use ai::{build_embedding_provider, EmbeddingProvider};
pub use config::{
    AiConfig, AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, ServerConfig,
    SmtpSecurity, StorageConfig,
};
pub use email::send_otp_email;
pub use pool::{init_db_pool, init_redis_client};
pub use queue::{
    get_job_status, list_dlq_entries, publish_ingestion_job, replay_dlq_entry,
    INGESTION_DLQ_STREAM, INGESTION_GROUP, INGESTION_STREAM,
};
pub use repositories::{
    badge_repository::{list_badges, list_user_badges, seed_default_badges_if_empty},
    book_repository::{
        chapter_dropoff, ensure_book_published, get_book_by_id, get_book_for_admin,
        get_book_status, get_chapter_book_id, get_chapter_by_number, get_chapter_number,
        get_offline_bundle, list_books, list_books_for_admin, search_books, set_book_status,
    },
    progress_repository::{get_active_progress, get_streak, record_heartbeat, update_progress},
    quote_repository::{
        get_chapter_recap, get_saved_quote_by_id, get_tldr_cache, list_saved_quotes, save_quote,
        save_quotes_batch, search_quotes_by_embedding, QuoteSaveOutcome, QuoteSearchRow,
        SavedQuoteDto,
    },
    user_repository::{
        activate_user_by_email, create_inactive_user, delete_user_by_id, find_user_by_email,
        find_user_by_id, update_inactive_credentials, update_user_password, update_user_profile,
    },
};
pub use security::{
    blacklist_access_token, generate_access_token, generate_access_token_issued_at,
    generate_and_store_otp, generate_refresh_token, hash_otp, hash_password, hash_password_async,
    invalidate_user_tokens, is_token_blacklisted, is_user_token_revoked, revoke_all_user_sessions,
    revoke_family_on_reuse, revoke_other_user_sessions, revoke_refresh_token, rotation_grace_key,
    store_refresh_token, validate_and_rotate_refresh_token, verify_access_token,
    verify_and_consume_otp, verify_password, verify_password_async, Claims, RefreshOutcome,
    DUMMY_ARGON2_HASH, ROTATION_GRACE_SECS,
};
pub use security::{
    issue_access_token, list_sessions, revoke_other_sessions_keeping, revoke_session,
    touch_session, SessionMeta, SessionRecord,
};
pub use storage::StorageService;
