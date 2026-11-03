//! Authentication and security primitives (Argon2id, OTP, JWT token rotation).

pub mod jwt;
pub mod otp;
pub mod password;

pub use jwt::{
    blacklist_access_token, generate_access_token, generate_access_token_issued_at,
    generate_refresh_token, invalidate_user_tokens, is_token_blacklisted, is_user_token_revoked,
    revoke_all_user_sessions, revoke_family_on_reuse, revoke_other_user_sessions,
    revoke_refresh_token, rotation_grace_key, store_refresh_token,
    validate_and_rotate_refresh_token, verify_access_token, Claims, RefreshOutcome,
    ROTATION_GRACE_SECS,
};
pub use jwt::{
    is_session_revoked, issue_access_token, list_sessions, revoke_other_sessions_keeping,
    revoke_session, revoke_session_tokens, touch_session, SessionMeta, SessionRecord,
};
pub use otp::{
    claim_code_quota, ensure_guesses_left, hash_otp, registration_key, start_registration,
    verify_registration, PendingRegistration,
};
pub use password::{hash_password, verify_password, Argon2Gate, DUMMY_ARGON2_HASH};
