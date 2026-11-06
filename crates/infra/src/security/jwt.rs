//! JWT token creation, verification, and Redis refresh token rotation.

use chrono::Utc;
use jsonwebtoken::{decode, decode_header, encode, DecodingKey, EncodingKey, Header, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

/// Audience of every access token this API issues and accepts.
pub const ACCESS_AUDIENCE: &str = "rotaria-api";

/// JOSE `typ` of an access token (RFC 9068), so it cannot pass for another
/// kind of JWT signed with the same key.
pub const ACCESS_TOKEN_TYPE: &str = "at+jwt";

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
    /// Session the token belongs to, the same across refreshes; signing a
    /// device out revokes every access token carrying it.
    pub sid: Uuid,
    /// Always [`ACCESS_AUDIENCE`].
    pub aud: String,
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
    issue_access_token(
        user_id,
        email,
        role,
        secret,
        expiry_minutes,
        issued_at,
        Uuid::new_v4(),
    )
    .map(|(t, _)| t)
}

/// Signs an access token for session `sid` and returns its `jti`, so the
/// session record of the refresh token it travels with can name it.
pub fn issue_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    secret: &str,
    expiry_minutes: u64,
    issued_at: usize,
    sid: Uuid,
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
        sid,
        aud: ACCESS_AUDIENCE.to_string(),
    };
    let header = Header {
        typ: Some(ACCESS_TOKEN_TYPE.to_string()),
        ..Header::default()
    };

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map(|token| (token, jti))
    .map_err(|e| AppError::Internal(format!("Failed to sign JWT access token: {e}")))
}

/// Verifies and decodes a signed JWT access token. Anything but `typ: at+jwt`
/// with `exp`, `aud`, and a session id is refused, so every accepted token
/// can be revoked per device; the client hears one generic reason and the
/// library's detail stays in the debug log.
pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let refuse = |reason: String| {
        tracing::debug!(target: "infra::jwt", reason, "Access token refused");
        AppError::Unauthorized("Invalid or expired access token".to_string())
    };
    let header = decode_header(token).map_err(|e| refuse(e.to_string()))?;
    if header.typ.as_deref() != Some(ACCESS_TOKEN_TYPE) {
        return Err(refuse(format!("unexpected typ {:?}", header.typ)));
    }
    let mut validation = Validation::default();
    validation.set_audience(&[ACCESS_AUDIENCE]);
    validation.set_required_spec_claims(&["exp", "aud"]);
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| refuse(e.to_string()))?;
    // The nil id is never issued, and no revocation key is ever written for it.
    if claims.sid.is_nil() {
        return Err(refuse("nil sid".to_string()));
    }
    Ok(claims)
}

/// Blacklists an access token by its JTI in Redis until expiration
pub async fn blacklist_access_token(
    redis: &mut crate::RedisConn,
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
    redis: &mut crate::RedisConn,
    jti: Uuid,
) -> Result<bool, AppError> {
    let redis_key = format!("blacklist:jti:{jti}");
    let exists: bool = redis
        .exists(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to query blacklist from Redis: {e}")))?;
    Ok(exists)
}

fn revoked_session_key(sid: Uuid) -> String {
    format!("revoked_sid:{sid}")
}

/// Revokes every access token of session `sid`, and any refresh of it, for
/// `ttl_seconds`; callers pass at least the access token lifetime.
pub async fn revoke_session_tokens(
    redis: &mut crate::RedisConn,
    sid: Uuid,
    ttl_seconds: u64,
) -> Result<(), AppError> {
    if sid.is_nil() || ttl_seconds == 0 {
        return Ok(());
    }
    let _: () = redis
        .set_ex(revoked_session_key(sid), "1", ttl_seconds)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to revoke session in Redis: {e}")))?;
    Ok(())
}

/// Whether the session an access token belongs to was signed out.
pub async fn is_session_revoked(redis: &mut crate::RedisConn, sid: Uuid) -> Result<bool, AppError> {
    redis
        .exists(revoked_session_key(sid))
        .await
        .map_err(|e| AppError::Internal(format!("Failed to query session revocation: {e}")))
}

/// Records a timestamp before which all issued tokens for a user are considered revoked
pub async fn invalidate_user_tokens(
    redis: &mut crate::RedisConn,
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
    redis: &mut crate::RedisConn,
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
    /// Session id every access token of this device carries; nil on records
    /// written before it existed.
    #[serde(default)]
    pub sid: Uuid,
}

/// Facts about the device a token pair is issued to.
#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub device: String,
    pub ip_prefix: String,
    pub jti: Uuid,
    pub sid: Uuid,
}

fn session_key(token_hash: &str) -> String {
    format!("session:{token_hash}")
}

async fn write_session_record(
    redis: &mut crate::RedisConn,
    token_hash: &str,
    record: &SessionRecord,
    ttl_seconds: u64,
) {
    // The record carries the session's sign-in time, which bounds its
    // lifetime; a lost write is logged, the token itself stays valid.
    if let Ok(json) = serde_json::to_string(record) {
        let written: Result<(), _> = redis
            .set_ex(session_key(token_hash), json, ttl_seconds)
            .await;
        if let Err(e) = written {
            tracing::warn!(target: "infra::jwt", error = %e, "Session record not written");
        }
    }
}

async fn read_session_record(
    redis: &mut crate::RedisConn,
    token_hash: &str,
) -> Option<SessionRecord> {
    let json: Option<String> = redis.get(session_key(token_hash)).await.unwrap_or(None);
    json.and_then(|j| serde_json::from_str(&j).ok())
}

/// Stores a refresh token mapped to a user ID in Redis with expiration in days,
/// plus the session record readers see on their shelf. Only the SHA-256 hash
/// is persisted: a Redis dump must not yield directly usable session tokens.
pub async fn store_refresh_token(
    redis: &mut crate::RedisConn,
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
            sid: meta.sid,
        },
        ttl_seconds,
    )
    .await;

    Ok(())
}

/// Points the session record of a refresh token at the access token just
/// issued with it and marks it seen now. The record keeps its TTL, which
/// rotation already capped at the session's absolute lifetime.
pub async fn touch_session(redis: &mut crate::RedisConn, token: &str, jti: Uuid) {
    let hash = token_hash(token);
    if let Some(mut record) = read_session_record(redis, &hash).await {
        record.jti = jti;
        record.last_seen_at = Utc::now().timestamp();
        if let Ok(json) = serde_json::to_string(&record) {
            let _: Result<(), _> = redis::cmd("SET")
                .arg(session_key(&hash))
                .arg(json)
                .arg("KEEPTTL")
                .query_async(redis)
                .await;
        }
    }
}

/// Every live session of a user, keyed by session id (the token hash).
pub async fn list_sessions(
    redis: &mut crate::RedisConn,
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

/// How long a session revocation must hold: as long as any access token of
/// the session, and as long as the refresh token could still be presented,
/// so a successor that slipped past the deletion is refused on refresh.
async fn revocation_ttl(redis: &mut crate::RedisConn, token_hash: &str, access_ttl: u64) -> u64 {
    let remaining: i64 = redis.ttl(session_key(token_hash)).await.unwrap_or(0);
    access_ttl.max(remaining.max(0) as u64)
}

/// Revokes one session of the user: its refresh token dies and the access
/// token last issued with it is blacklisted. False when the id is not theirs.
pub async fn revoke_session(
    redis: &mut crate::RedisConn,
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
        let ttl = revocation_ttl(redis, session_id, access_ttl_seconds).await;
        revoke_session_tokens(redis, record.sid, ttl).await?;
    }
    let _: Result<(), _> = redis.del(format!("refresh_token:{session_id}")).await;
    let _: Result<(), _> = redis.del(session_key(session_id)).await;
    let _: Result<(), _> = redis.srem(&user_set_key, session_id).await;
    Ok(true)
}

/// Signs every other device out while the caller (identified by the access
/// token it holds) stays signed in; no global revocation stamp is written.
pub async fn revoke_other_sessions_keeping(
    redis: &mut crate::RedisConn,
    user_id: Uuid,
    current_jti: Uuid,
    current_sid: Uuid,
    access_ttl_seconds: u64,
) -> Result<usize, AppError> {
    let mut revoked = 0;
    for (hash, record) in list_sessions(redis, user_id).await {
        if record
            .as_ref()
            .is_some_and(|r| r.jti == current_jti || r.sid == current_sid)
        {
            continue;
        }
        if revoke_session(redis, user_id, &hash, access_ttl_seconds).await? {
            revoked += 1;
        }
    }
    Ok(revoked)
}

/// Id of the session a refresh token belongs to (its SHA-256), as the
/// session list names it; also keys the session record and rate limits.
pub fn session_id(token: &str) -> String {
    token_hash(token)
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
        /// Session the successor belongs to; access tokens issued with it
        /// carry this id.
        sid: Uuid,
    },
    /// A token rotated out before the grace window was presented again.
    Replayed { user_id: Uuid },
    /// A live token whose session is older than the absolute lifetime; it was
    /// not rotated and the caller ends the session.
    Expired { user_id: Uuid },
}

/// Consumes the old token and stores its successor, with the successor's
/// session record, in one step: a family revocation running at the same
/// moment either sees the old token or the successor, never neither, and a
/// second tab taking the grace path always finds the session it belongs to.
const ROTATE_SCRIPT: &str = r"
local owner = redis.call('GETDEL', KEYS[1])
if not owner then return false end
local family = 'user_refresh_tokens:' .. owner
redis.call('SREM', family, ARGV[1])
redis.call('SET', 'rotated_refresh_token:' .. ARGV[1], owner, 'EX', ARGV[4])
redis.call('SET', 'rotation_grace:' .. ARGV[1], ARGV[2], 'EX', ARGV[5])
redis.call('SET', 'refresh_token:' .. ARGV[3], owner, 'EX', ARGV[4])
redis.call('SADD', family, ARGV[3])
-- A successor capped by its session's absolute lifetime must not shorten the
-- set that every other device of the reader is listed in.
if redis.call('TTL', family) < tonumber(ARGV[4]) then
  redis.call('EXPIRE', family, ARGV[4])
end
if ARGV[6] ~= '' then
  redis.call('SET', 'session:' .. ARGV[3], ARGV[6], 'EX', ARGV[4])
end
redis.call('DEL', 'session:' .. ARGV[1])
return owner
";

/// Deletes every refresh token of a family and the family set at once.
const REVOKE_FAMILY_SCRIPT: &str = r"
local hashes = redis.call('SMEMBERS', KEYS[1])
for _, hash in ipairs(hashes) do
  redis.call('DEL', 'refresh_token:' .. hash)
end
redis.call('DEL', KEYS[1])
return #hashes
";

/// Consumes an active refresh token atomically and issues its successor.
/// One script guarantees a single winner among concurrent presenters;
/// losers are resolved against the rotation record (grace successor or
/// replay).
pub async fn validate_and_rotate_refresh_token(
    redis: &mut crate::RedisConn,
    old_token: &str,
    expiry_days: u64,
    session_max_days: u64,
) -> Result<RefreshOutcome, AppError> {
    let old_hash = token_hash(old_token);
    let new_token = generate_refresh_token();
    let new_hash = token_hash(&new_token);
    let previous = read_session_record(redis, &old_hash).await;
    let now = Utc::now().timestamp();
    let session_left = previous
        .as_ref()
        .map(|record| record.created_at + (session_max_days * 86400) as i64 - now);
    if let (Some(record), Some(left)) = (&previous, session_left) {
        if left <= 0 {
            return Ok(RefreshOutcome::Expired {
                user_id: record.user_id,
            });
        }
    }
    // A successor never outlives the session's absolute lifetime.
    let ttl_seconds = session_left.map_or(expiry_days * 86400, |left| {
        (expiry_days * 86400).min(left as u64)
    });
    // The device and its session stay the same across a rotation; only the
    // token changes.
    let successor = previous.map(|previous| SessionRecord {
        last_seen_at: Utc::now().timestamp(),
        sid: if previous.sid.is_nil() {
            Uuid::new_v4()
        } else {
            previous.sid
        },
        ..previous
    });
    let successor_json = successor
        .as_ref()
        .and_then(|record| serde_json::to_string(record).ok())
        .unwrap_or_default();
    let owner: Option<String> = redis::Script::new(ROTATE_SCRIPT)
        .key(refresh_key(old_token))
        .arg(&old_hash)
        .arg(&new_token)
        .arg(&new_hash)
        .arg(ttl_seconds)
        .arg(ROTATION_GRACE_SECS)
        .arg(successor_json)
        .invoke_async(redis)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to rotate refresh token in Redis: {e}")))?;

    let Some(owner) = owner else {
        return resolve_consumed_token(redis, &old_hash).await;
    };

    let user_id = Uuid::parse_str(&owner).map_err(|_| {
        AppError::Unauthorized("Corrupted user ID in refresh token session".to_string())
    })?;

    // A token without a session record predates records; it starts a fresh
    // session here.
    let sid = match successor {
        Some(record) => record.sid,
        None => {
            let now = Utc::now().timestamp();
            let record = SessionRecord {
                user_id,
                device: String::new(),
                ip_prefix: String::new(),
                created_at: now,
                last_seen_at: now,
                jti: Uuid::nil(),
                sid: Uuid::new_v4(),
            };
            write_session_record(redis, &new_hash, &record, ttl_seconds).await;
            record.sid
        }
    };

    Ok(RefreshOutcome::Rotated {
        user_id,
        refresh_token: new_token,
        sid,
    })
}

/// A token absent from the live set was never issued, was rotated inside the
/// grace window (hand back the same successor), or was rotated earlier.
async fn resolve_consumed_token(
    redis: &mut crate::RedisConn,
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
    let Some(refresh_token) = successor else {
        return Ok(RefreshOutcome::Replayed { user_id });
    };
    // Revoke-all or a logout inside the window killed the successor; handing
    // it back would mint access tokens for a session that no longer exists.
    let successor_live: bool = redis
        .exists(refresh_key(&refresh_token))
        .await
        .map_err(|e| AppError::Internal(format!("Failed to check refresh successor: {e}")))?;
    if !successor_live {
        return Err(AppError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ));
    }
    let sid = read_session_record(redis, &token_hash(&refresh_token))
        .await
        .map(|record| record.sid)
        .unwrap_or_default();
    Ok(RefreshOutcome::Rotated {
        user_id,
        refresh_token,
        sid,
    })
}

/// Revokes an active refresh token and blacklists the access token last
/// issued with it, so a logout that carries no `Authorization` header still
/// ends the whole session.
pub async fn revoke_refresh_token(
    redis: &mut crate::RedisConn,
    token: &str,
    access_ttl_seconds: u64,
) -> Result<(), AppError> {
    let redis_key = refresh_key(token);
    let hash = token_hash(token);
    let user_id_str: Option<String> = redis.get(&redis_key).await.unwrap_or(None);
    if let Some(record) = read_session_record(redis, &hash).await {
        blacklist_access_token(redis, record.jti, access_ttl_seconds).await?;
        let ttl = revocation_ttl(redis, &hash, access_ttl_seconds).await;
        revoke_session_tokens(redis, record.sid, ttl).await?;
    }
    let _: () = redis
        .del(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to revoke refresh token: {e}")))?;
    let _: Result<(), _> = redis.del(session_key(&hash)).await;

    if let Some(uid_str) = user_id_str {
        if let Ok(uid) = Uuid::parse_str(&uid_str) {
            let user_set_key = format!("user_refresh_tokens:{uid}");
            let _: Result<(), _> = redis.srem(&user_set_key, &hash).await;
        }
    }

    Ok(())
}

/// Revokes all active refresh tokens and access tokens for the specified user
pub async fn revoke_all_user_sessions(
    redis: &mut crate::RedisConn,
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
    redis: &mut crate::RedisConn,
    user_id: Uuid,
) -> Result<(), AppError> {
    let _: i64 = redis::Script::new(REVOKE_FAMILY_SCRIPT)
        .key(format!("user_refresh_tokens:{user_id}"))
        .invoke_async(redis)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to revoke refresh family: {e}")))?;
    Ok(())
}

/// Reuse of a rotated (dead) refresh token signals theft: kill the whole
/// family so the attacker's copy dies with the victim's session.
pub async fn revoke_family_on_reuse(
    redis: &mut crate::RedisConn,
    user_id: Uuid,
    access_token_max_expiry_secs: u64,
) -> Result<(), AppError> {
    revoke_refresh_family(redis, user_id).await?;
    invalidate_user_tokens(redis, user_id, access_token_max_expiry_secs).await
}

/// Revokes all refresh tokens for a user except the specified one
pub async fn revoke_other_user_sessions(
    redis: &mut crate::RedisConn,
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

    const SECRET: &str = "super-secret-test-jwt-key-minimum-32-chars!";

    fn at_jwt() -> Header {
        Header {
            typ: Some(ACCESS_TOKEN_TYPE.into()),
            ..Header::default()
        }
    }

    /// Every claim an access token carries, as JSON so a test can drop one.
    fn full_claims() -> serde_json::Value {
        let now = Utc::now().timestamp();
        serde_json::json!({
            "sub": Uuid::new_v4(), "email": "reader@example.com", "role": "reader",
            "exp": now + 600, "iat": now, "jti": Uuid::new_v4(),
            "sid": Uuid::new_v4(), "aud": ACCESS_AUDIENCE,
        })
    }

    fn with(claim: &str, value: serde_json::Value) -> serde_json::Value {
        let mut claims = full_claims();
        claims[claim] = value;
        claims
    }

    fn without(claim: &str) -> serde_json::Value {
        let mut claims = full_claims();
        claims.as_object_mut().unwrap().remove(claim);
        claims
    }

    fn signed(header: &Header, claims: &serde_json::Value) -> String {
        encode(header, claims, &EncodingKey::from_secret(SECRET.as_bytes())).unwrap()
    }

    #[test]
    fn access_tokens_carry_audience_and_type() {
        let token =
            generate_access_token(Uuid::new_v4(), "r@example.com", "reader", SECRET, 15).unwrap();
        assert_eq!(
            decode_header(&token).unwrap().typ.as_deref(),
            Some(ACCESS_TOKEN_TYPE)
        );
        let claims = verify_access_token(&token, SECRET).unwrap();
        assert_eq!(claims.aud, ACCESS_AUDIENCE);
        assert!(!claims.sid.is_nil());
    }

    #[test]
    fn a_token_for_another_audience_or_type_is_refused() {
        assert!(verify_access_token(&signed(&at_jwt(), &full_claims()), SECRET).is_ok());
        let other_aud = with("aud", serde_json::json!("other-api"));
        assert!(verify_access_token(&signed(&at_jwt(), &other_aud), SECRET).is_err());
        for typ in [Some("JWT"), Some("id+jwt"), None] {
            let header = Header {
                typ: typ.map(str::to_string),
                ..Header::default()
            };
            assert!(
                verify_access_token(&signed(&header, &full_claims()), SECRET).is_err(),
                "typ {typ:?}"
            );
        }
    }

    #[test]
    fn a_token_without_exp_aud_or_session_is_refused() {
        for claim in ["exp", "aud", "sid"] {
            assert!(
                verify_access_token(&signed(&at_jwt(), &without(claim)), SECRET).is_err(),
                "{claim}"
            );
        }
        for sid in [serde_json::json!(Uuid::nil()), serde_json::json!(null)] {
            assert!(
                verify_access_token(&signed(&at_jwt(), &with("sid", sid.clone())), SECRET).is_err(),
                "sid {sid}"
            );
        }
    }

    #[test]
    fn a_token_whose_exp_is_not_a_number_is_refused() {
        // A non-numeric exp must never read as "absent", and a non-numeric
        // nbf must not rescue an expired token. nbf itself is not validated:
        // no token here carries one.
        let now = Utc::now().timestamp();
        for claims in [with("exp", serde_json::json!("9999999999")), {
            let mut claims = with("exp", serde_json::json!(now - 3600));
            claims["nbf"] = serde_json::json!("0");
            claims
        }] {
            assert!(
                verify_access_token(&signed(&at_jwt(), &claims), SECRET).is_err(),
                "{claims}"
            );
        }
    }

    #[test]
    fn a_refusal_names_no_library_error() {
        let expired = {
            let now = Utc::now().timestamp() as usize;
            issue_access_token(
                Uuid::new_v4(),
                "r@example.com",
                "reader",
                SECRET,
                0,
                now - 3600,
                Uuid::new_v4(),
            )
            .unwrap()
            .0
        };
        for token in [
            expired,
            "not.a.jwt".to_string(),
            signed(&at_jwt(), &with("aud", serde_json::json!("x"))),
            signed(&at_jwt(), &without("sid")),
        ] {
            match verify_access_token(&token, SECRET) {
                Err(AppError::Unauthorized(message)) => {
                    assert_eq!(message, "Invalid or expired access token")
                }
                other => panic!("expected a generic 401, got {other:?}"),
            }
        }
    }
}
