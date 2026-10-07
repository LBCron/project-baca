//! JWT token creation, verification, and Redis refresh token rotation.

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

/// JWT token payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (User ID)
    pub sub: Uuid,
    /// User email address
    pub email: String,
    /// User role (reader, admin, guest)
    pub role: String,
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
    /// Unique JWT Token ID
    pub jti: Uuid,
}

/// Generates a signed JWT access token with the specified expiration in minutes
pub fn generate_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    secret: &str,
    expiry_minutes: u64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    generate_access_token_issued_at(user_id, email, role, secret, expiry_minutes, now)
}

/// Signs an access token with an explicit `iat`. Revocation compares whole
/// seconds (`iat <= revoked_before`), so a pair minted right after revoking
/// a user's sessions is stamped one second ahead to outlive the cut.
pub fn generate_access_token_issued_at(
    user_id: Uuid,
    email: &str,
    role: &str,
    secret: &str,
    expiry_minutes: u64,
    issued_at: usize,
) -> Result<String, AppError> {
    issue_access_token(user_id, email, role, secret, expiry_minutes, issued_at).map(|(t, _)| t)
}

/// Signs an access token and returns its `jti`, so the session record of the
/// refresh token it travels with can name it.
pub fn issue_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    secret: &str,
    expiry_minutes: u64,
    issued_at: usize,
) -> Result<(String, Uuid), AppError> {
    let exp = issued_at + (expiry_minutes as usize * 60);
    let jti = Uuid::new_v4();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp,
        iat: issued_at,
        jti,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map(|token| (token, jti))
    .map_err(|e| AppError::Internal(format!("Failed to sign JWT access token: {e}")))
}

/// Verifies and decodes a signed JWT access token
pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid or expired access token: {e}")))
}

/// Blacklists an access token by its JTI in Redis until expiration
pub async fn blacklist_access_token(
    redis: &mut redis::aio::MultiplexedConnection,
    jti: Uuid,
    ttl_seconds: u64,
) -> Result<(), AppError> {
    if ttl_seconds == 0 {
        return Ok(());
    }
    let redis_key = format!("blacklist:jti:{jti}");
    let _: () = redis
        .set_ex(&redis_key, "1", ttl_seconds)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to blacklist token in Redis: {e}")))?;
    Ok(())
}

/// Checks whether an access token JTI is in the Redis blacklist
pub async fn is_token_blacklisted(
    redis: &mut redis::aio::MultiplexedConnection,
    jti: Uuid,
) -> Result<bool, AppError> {
    let redis_key = format!("blacklist:jti:{jti}");
    let exists: bool = redis
        .exists(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to query blacklist from Redis: {e}")))?;
    Ok(exists)
}

/// Records a timestamp before which all issued tokens for a user are considered revoked
pub async fn invalidate_user_tokens(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    ttl_seconds: u64,
) -> Result<(), AppError> {
    let redis_key = format!("user_revoked_before:{user_id}");
    let now = Utc::now().timestamp();
    let _: () = redis
        .set_ex(&redis_key, now, ttl_seconds)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to record token revocation in Redis: {e}"))
        })?;
    Ok(())
}

/// Checks whether a token's issuance time (iat) is prior to the user's revocation timestamp
pub async fn is_user_token_revoked(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    token_iat: usize,
) -> Result<bool, AppError> {
    let redis_key = format!("user_revoked_before:{user_id}");
    let revoked_before_str: Option<String> = redis.get(&redis_key).await.map_err(|e| {
        AppError::Internal(format!("Failed to check user revocation in Redis: {e}"))
    })?;

    if let Some(s) = revoked_before_str {
        if let Ok(revoked_timestamp) = s.parse::<i64>() {
            if (token_iat as i64) <= revoked_timestamp {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Generates a cryptographically random refresh token
pub fn generate_refresh_token() -> String {
    format!("rt_{}_{:016x}", Uuid::new_v4(), rand::random::<u64>())
}

/// What a reader sees about one signed-in device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub user_id: Uuid,
    pub device: String,
    pub ip_prefix: String,
    pub created_at: i64,
    pub last_seen_at: i64,
    /// `jti` of the access token last issued with this refresh token.
    pub jti: Uuid,
}

/// Facts about the device a token pair is issued to.
#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub device: String,
    pub ip_prefix: String,
    pub jti: Uuid,
}

fn session_key(token_hash: &str) -> String {
    format!("session:{token_hash}")
}

async fn write_session_record(
    redis: &mut redis::aio::MultiplexedConnection,
    token_hash: &str,
    record: &SessionRecord,
    ttl_seconds: u64,
) {
    if let Ok(json) = serde_json::to_string(record) {
        let _: Result<(), _> = redis
            .set_ex(session_key(token_hash), json, ttl_seconds)
            .await;
    }
}

async fn read_session_record(
    redis: &mut redis::aio::MultiplexedConnection,
    token_hash: &str,
) -> Option<SessionRecord> {
    let json: Option<String> = redis.get(session_key(token_hash)).await.unwrap_or(None);
    json.and_then(|j| serde_json::from_str(&j).ok())
}

/// Stores a refresh token mapped to a user ID in Redis with expiration in days,
/// plus the session record readers see on their shelf. Only the SHA-256 hash
/// is persisted: a Redis dump must not yield directly usable session tokens.
pub async fn store_refresh_token(
    redis: &mut redis::aio::MultiplexedConnection,
    token: &str,
    user_id: Uuid,
    expiry_days: u64,
    meta: SessionMeta,
) -> Result<(), AppError> {
    let redis_key = refresh_key(token);
    let user_set_key = format!("user_refresh_tokens:{user_id}");
    let ttl_seconds = expiry_days * 86400;

    let _: () = redis
        .set_ex(&redis_key, user_id.to_string(), ttl_seconds)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to persist refresh token in Redis: {e}"))
        })?;

    let hash = token_hash(token);
    let _: Result<(), _> = redis.sadd(&user_set_key, &hash).await;
    let _: Result<(), _> = redis.expire(&user_set_key, ttl_seconds as i64).await;
    let now = Utc::now().timestamp();
    write_session_record(
        redis,
        &hash,
        &SessionRecord {
            user_id,
            device: meta.device,
            ip_prefix: meta.ip_prefix,
            created_at: now,
            last_seen_at: now,
            jti: meta.jti,
        },
        ttl_seconds,
    )
    .await;

    Ok(())
}

/// Points the session record of a refresh token at the access token just
/// issued with it and marks it seen now.
pub async fn touch_session(
    redis: &mut redis::aio::MultiplexedConnection,
    token: &str,
    jti: Uuid,
    expiry_days: u64,
) {
    let hash = token_hash(token);
    if let Some(mut record) = read_session_record(redis, &hash).await {
        record.jti = jti;
        record.last_seen_at = Utc::now().timestamp();
        write_session_record(redis, &hash, &record, expiry_days * 86400).await;
    }
}

/// Every live session of a user, keyed by session id (the token hash).
pub async fn list_sessions(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
) -> Vec<(String, Option<SessionRecord>)> {
    let user_set_key = format!("user_refresh_tokens:{user_id}");
    let hashes: Vec<String> = redis.smembers(&user_set_key).await.unwrap_or_default();
    let mut sessions = Vec::with_capacity(hashes.len());
    for hash in hashes {
        let live: bool = redis
            .exists(format!("refresh_token:{hash}"))
            .await
            .unwrap_or(false);
        if !live {
            let _: Result<(), _> = redis.srem(&user_set_key, &hash).await;
            continue;
        }
        let record = read_session_record(redis, &hash).await;
        sessions.push((hash, record));
    }
    sessions
}

/// Revokes one session of the user: its refresh token dies and the access
/// token last issued with it is blacklisted. False when the id is not theirs.
pub async fn revoke_session(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    session_id: &str,
    access_ttl_seconds: u64,
) -> Result<bool, AppError> {
    let user_set_key = format!("user_refresh_tokens:{user_id}");
    let owned: bool = redis
        .sismember(&user_set_key, session_id)
        .await
        .map_err(|e| AppError::Internal(format!("Session lookup failed: {e}")))?;
    if !owned {
        return Ok(false);
    }
    if let Some(record) = read_session_record(redis, session_id).await {
        blacklist_access_token(redis, record.jti, access_ttl_seconds).await?;
    }
    let _: Result<(), _> = redis.del(format!("refresh_token:{session_id}")).await;
    let _: Result<(), _> = redis.del(session_key(session_id)).await;
    let _: Result<(), _> = redis.srem(&user_set_key, session_id).await;
    Ok(true)
}

/// Signs every other device out while the caller (identified by the access
/// token it holds) stays signed in; no global revocation stamp is written.
pub async fn revoke_other_sessions_keeping(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    current_jti: Uuid,
    access_ttl_seconds: u64,
) -> Result<usize, AppError> {
    let mut revoked = 0;
    for (hash, record) in list_sessions(redis, user_id).await {
        if record.as_ref().map(|r| r.jti) == Some(current_jti) {
            continue;
        }
        if revoke_session(redis, user_id, &hash, access_ttl_seconds).await? {
            revoked += 1;
        }
    }
    Ok(revoked)
}

fn token_hash(token: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn refresh_key(token: &str) -> String {
    format!("refresh_token:{}", token_hash(token))
}

fn rotated_key(token_hash: &str) -> String {
    format!("rotated_refresh_token:{token_hash}")
}

/// Redis key holding the successor of a just-rotated token for
/// [`ROTATION_GRACE_SECS`]. Public so tests can expire the window early.
pub fn rotation_grace_key(token: &str) -> String {
    format!("rotation_grace:{}", token_hash(token))
}

/// Window after a rotation during which the consumed token still yields the
/// same successor. Two tabs or a retried request refreshing at once must not
/// be mistaken for theft; beyond it a replay revokes the whole family.
pub const ROTATION_GRACE_SECS: u64 = 30;

/// Result of presenting a refresh token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshOutcome {
    /// Token accepted: the owner and the refresh token to hand back.
    Rotated {
        user_id: Uuid,
        refresh_token: String,
    },
    /// A token rotated out before the grace window was presented again.
    Replayed { user_id: Uuid },
}

/// Consumes an active refresh token atomically and issues its successor.
/// GETDEL guarantees a single winner among concurrent presenters; losers are
/// resolved against the rotation record (grace successor or replay).
pub async fn validate_and_rotate_refresh_token(
    redis: &mut redis::aio::MultiplexedConnection,
    old_token: &str,
    expiry_days: u64,
) -> Result<RefreshOutcome, AppError> {
    let old_hash = token_hash(old_token);
    let owner: Option<String> = redis::cmd("GETDEL")
        .arg(refresh_key(old_token))
        .query_async(redis)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to consume refresh token in Redis: {e}"))
        })?;

    let Some(owner) = owner else {
        return resolve_consumed_token(redis, &old_hash).await;
    };

    let user_id = Uuid::parse_str(&owner).map_err(|_| {
        AppError::Unauthorized("Corrupted user ID in refresh token session".to_string())
    })?;

    let user_set_key = format!("user_refresh_tokens:{user_id}");
    let _: Result<(), _> = redis.srem(&user_set_key, &old_hash).await;

    let new_token = generate_refresh_token();
    let _: Result<(), _> = redis
        .set_ex(
            rotated_key(&old_hash),
            user_id.to_string(),
            expiry_days * 86400,
        )
        .await;
    let _: Result<(), _> = redis
        .set_ex(
            rotation_grace_key(old_token),
            &new_token,
            ROTATION_GRACE_SECS,
        )
        .await;
    // The device stays the same across a rotation; only the token changes.
    let previous = read_session_record(redis, &old_hash).await;
    let meta = previous
        .as_ref()
        .map(|p| SessionMeta {
            device: p.device.clone(),
            ip_prefix: p.ip_prefix.clone(),
            jti: p.jti,
        })
        .unwrap_or_default();
    store_refresh_token(redis, &new_token, user_id, expiry_days, meta).await?;
    if let Some(previous) = previous {
        let new_hash = token_hash(&new_token);
        if let Some(mut record) = read_session_record(redis, &new_hash).await {
            record.created_at = previous.created_at;
            write_session_record(redis, &new_hash, &record, expiry_days * 86400).await;
        }
    }
    let _: Result<(), _> = redis.del(session_key(&old_hash)).await;

    Ok(RefreshOutcome::Rotated {
        user_id,
        refresh_token: new_token,
    })
}

/// A token absent from the live set was never issued, was rotated inside the
/// grace window (hand back the same successor), or was rotated earlier.
async fn resolve_consumed_token(
    redis: &mut redis::aio::MultiplexedConnection,
    old_hash: &str,
) -> Result<RefreshOutcome, AppError> {
    let rotated_owner: Option<String> = redis.get(rotated_key(old_hash)).await.unwrap_or(None);
    let Some(user_id) = rotated_owner.and_then(|s| Uuid::parse_str(&s).ok()) else {
        return Err(AppError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ));
    };
    let successor: Option<String> = redis
        .get(format!("rotation_grace:{old_hash}"))
        .await
        .unwrap_or(None);
    Ok(match successor {
        Some(refresh_token) => RefreshOutcome::Rotated {
            user_id,
            refresh_token,
        },
        None => RefreshOutcome::Replayed { user_id },
    })
}

/// Revokes an active refresh token from Redis
pub async fn revoke_refresh_token(
    redis: &mut redis::aio::MultiplexedConnection,
    token: &str,
) -> Result<(), AppError> {
    let redis_key = refresh_key(token);
    let user_id_str: Option<String> = redis.get(&redis_key).await.unwrap_or(None);
    let _: () = redis
        .del(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to revoke refresh token: {e}")))?;

    if let Some(uid_str) = user_id_str {
        if let Ok(uid) = Uuid::parse_str(&uid_str) {
            let user_set_key = format!("user_refresh_tokens:{uid}");
            let _: Result<(), _> = redis.srem(&user_set_key, token_hash(token)).await;
        }
    }

    Ok(())
}

/// Revokes all active refresh tokens and access tokens for the specified user
pub async fn revoke_all_user_sessions(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    access_token_max_expiry_secs: u64,
) -> Result<(), AppError> {
    revoke_refresh_family(redis, user_id).await?;

    // Invalidate all outstanding access tokens issued before this instant
    invalidate_user_tokens(redis, user_id, access_token_max_expiry_secs).await?;

    Ok(())
}

/// Deletes every refresh token in the user's family without touching access
/// tokens. Shared by full revocation and theft-triggered family revocation.
async fn revoke_refresh_family(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
) -> Result<(), AppError> {
    let user_set_key = format!("user_refresh_tokens:{user_id}");
    let hashes: Vec<String> = redis.smembers(&user_set_key).await.unwrap_or_default();
    for hash in hashes {
        let token_key = format!("refresh_token:{hash}");
        let _: Result<(), _> = redis.del(&token_key).await;
    }
    let _: Result<(), _> = redis.del(&user_set_key).await;
    Ok(())
}

/// Reuse of a rotated (dead) refresh token signals theft: kill the whole
/// family so the attacker's copy dies with the victim's session.
pub async fn revoke_family_on_reuse(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    access_token_max_expiry_secs: u64,
) -> Result<(), AppError> {
    revoke_refresh_family(redis, user_id).await?;
    invalidate_user_tokens(redis, user_id, access_token_max_expiry_secs).await
}

/// Revokes all refresh tokens for a user except the specified one
pub async fn revoke_other_user_sessions(
    redis: &mut redis::aio::MultiplexedConnection,
    user_id: Uuid,
    keep_token: Option<&str>,
) -> Result<(), AppError> {
    let keep_hash = keep_token.map(token_hash);
    let user_set_key = format!("user_refresh_tokens:{user_id}");
    let hashes: Vec<String> = redis.smembers(&user_set_key).await.unwrap_or_default();
    for hash in hashes {
        if let Some(ref keep) = keep_hash {
            if hash == *keep {
                continue;
            }
        }
        let token_key = format!("refresh_token:{hash}");
        let _: Result<(), _> = redis.del(&token_key).await;
        let _: Result<(), _> = redis.srem(&user_set_key, &hash).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_verification() {
        let user_id = Uuid::new_v4();
        let email = "reader@example.com";
        let role = "reader";
        let secret = "super-secret-test-jwt-key-minimum-32-chars!";

        let token = generate_access_token(user_id, email, role, secret, 15)
            .expect("Token generation should succeed");

        let claims =
            verify_access_token(&token, secret).expect("Token verification should succeed");
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, role);
        assert!(!claims.jti.is_nil());
    }

    #[test]
    fn test_jwt_invalid_secret_rejection() {
        let user_id = Uuid::new_v4();
        let secret = "secret1-for-signing-this-token-here!";
        let wrong_secret = "secret2-different-from-signing-key!";

        let token =
            generate_access_token(user_id, "user@test.local", "reader", secret, 15).unwrap();
        let result = verify_access_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_refresh_token_format() {
        let token = generate_refresh_token();
        assert!(token.starts_with("rt_"));
        assert!(token.len() > 30);
    }
}
