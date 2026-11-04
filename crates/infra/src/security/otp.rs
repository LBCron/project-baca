//! Email OTP registrations kept in Redis until the code comes back. Each
//! signup is its own registration under a random id, holding the code hash
//! with the password and name its registrant chose, so a code only ever
//! activates the password of whoever asked for it. Guessing is bounded per
//! code (three misses lock it for five minutes) and per address over a day,
//! and the codes mailed to one address are capped per hour and per day.

use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::AppError;
use uuid::Uuid;

const OTP_TTL_SECONDS: u64 = 600; // 10 minutes
const MAX_OTP_ATTEMPTS: i64 = 3;
const OTP_LOCK_SECONDS: i64 = 300; // 5-minute cooldown after exhausting attempts
const DAY_SECONDS: u64 = 86_400;
/// Guesses per address per day, across every code it is sent.
const MAX_DAILY_GUESSES: i64 = 10;
/// Codes mailed to one address per hour and per day.
const MAX_CODES_PER_HOUR: i64 = 3;
const MAX_CODES_PER_DAY: i64 = 10;

/// A signup waiting for its code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRegistration {
    pub code_hash: String,
    pub display_name: String,
    pub password_hash: String,
}

/// Redis key of one pending registration; it lives as long as its code.
pub fn registration_key(email: &str, id: Uuid) -> String {
    format!("otp:{email}:{id}")
}

fn attempts_key(email: &str, id: Uuid) -> String {
    format!("otp:attempts:{email}:{id}")
}

fn guesses_key(email: &str) -> String {
    format!("otp:guesses:{email}")
}

fn redis_error(context: &str, e: redis::RedisError) -> AppError {
    AppError::Internal(format!("{context}: {e}"))
}

/// Computes SHA-256 hex digest of OTP string
pub fn hash_otp(otp: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(otp.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generates a cryptographically secure 6-digit numeric OTP string
pub fn generate_numeric_otp() -> String {
    let mut rng = rand::rngs::OsRng;
    let otp: u32 = rng.gen_range(100000..=999999);
    format!("{:06}", otp)
}

/// Refuses a new code while the address has spent its daily guesses, so
/// asking for fresh codes never buys more attempts.
pub async fn ensure_guesses_left(
    redis: &mut crate::RedisConn,
    email: &str,
) -> Result<(), AppError> {
    let key = guesses_key(email);
    let used: Option<i64> = redis
        .get(&key)
        .await
        .map_err(|e| redis_error("Redis read failed for OTP guesses", e))?;
    if used.unwrap_or(0) < MAX_DAILY_GUESSES {
        return Ok(());
    }
    let ttl: i64 = redis
        .ttl(&key)
        .await
        .map_err(|e| redis_error("Redis TTL failed for OTP guesses", e))?;
    Err(AppError::RateLimited {
        retry_after: ttl.max(1) as u64,
    })
}

fn code_quota_keys(email: &str) -> [(String, u64, i64); 2] {
    [
        (format!("otp:sent:3600:{email}"), 3600, MAX_CODES_PER_HOUR),
        (
            format!("otp:sent:{DAY_SECONDS}:{email}"),
            DAY_SECONDS,
            MAX_CODES_PER_DAY,
        ),
    ]
}

/// Gives back the code counted for `email` when no mail went out after all.
pub async fn release_code_quota(redis: &mut crate::RedisConn, email: &str) {
    let keys = code_quota_keys(email).map(|(key, _, _)| key);
    if let Err(e) = crate::release_hits(redis, &keys).await {
        tracing::warn!(target: "infra::otp", error = %e, "Could not give back an unsent code");
    }
}

/// Counts one code mailed to `email` against its hourly and daily caps,
/// refusing before anything is sent once either is spent.
pub async fn claim_code_quota(redis: &mut crate::RedisConn, email: &str) -> Result<(), AppError> {
    for (key, window, cap) in code_quota_keys(email) {
        let (sent, ttl) = crate::hit_window(redis, &key, window)
            .await
            .map_err(|e| redis_error("Redis count failed for OTP mail", e))?;
        if sent > cap {
            return Err(AppError::RateLimited {
                retry_after: ttl.max(1) as u64,
            });
        }
    }
    Ok(())
}

/// Stores a new registration and returns its id with the code to mail.
pub async fn start_registration(
    redis: &mut crate::RedisConn,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> Result<(Uuid, String), AppError> {
    let id = Uuid::new_v4();
    let otp = generate_numeric_otp();
    let record = serde_json::to_string(&PendingRegistration {
        code_hash: hash_otp(&otp),
        display_name: display_name.to_string(),
        password_hash: password_hash.to_string(),
    })
    .map_err(|e| AppError::Internal(format!("Failed to serialize registration: {e}")))?;
    let _: () = redis
        .set_ex(registration_key(email, id), record, OTP_TTL_SECONDS)
        .await
        .map_err(|e| redis_error("Redis write failed for registration", e))?;
    Ok((id, otp))
}

/// Checks a code against its registration and hands the registration over
/// once, deleting it. `None` for a wrong code or an unknown registration.
///
/// Both counters are taken with an atomic `INCR` before the hash is
/// compared, so parallel guesses each take their own slot. The lock of the
/// code is checked first: guesses at a locked code are refused without
/// spending the address's daily budget. Of two right answers at once only
/// the one that deletes the record wins.
pub async fn verify_registration(
    redis: &mut crate::RedisConn,
    email: &str,
    id: Uuid,
    otp: &str,
) -> Result<Option<PendingRegistration>, AppError> {
    let key = registration_key(email, id);
    let record: Option<String> = redis
        .get(&key)
        .await
        .map_err(|e| redis_error("Redis get failed for registration", e))?;
    let Some(record) = record else {
        return Ok(None);
    };
    let Ok(registration) = serde_json::from_str::<PendingRegistration>(&record) else {
        let _: Result<(), _> = redis.del(&key).await;
        return Ok(None);
    };

    let (attempt, ttl) = crate::hit_window(redis, &attempts_key(email, id), OTP_TTL_SECONDS)
        .await
        .map_err(|e| redis_error("Redis attempt count failed for OTP", e))?;
    if attempt > MAX_OTP_ATTEMPTS {
        return Err(AppError::RateLimited {
            retry_after: ttl.max(1) as u64,
        });
    }
    let (guesses, guesses_ttl) = crate::hit_window(redis, &guesses_key(email), DAY_SECONDS)
        .await
        .map_err(|e| redis_error("Redis guess count failed for OTP", e))?;
    if guesses > MAX_DAILY_GUESSES {
        return Err(AppError::RateLimited {
            retry_after: guesses_ttl.max(1) as u64,
        });
    }

    if registration.code_hash == hash_otp(otp) {
        let removed: i64 = redis
            .del(&key)
            .await
            .map_err(|e| redis_error("Redis delete failed for registration", e))?;
        let _: Result<(), _> = redis.del(attempts_key(email, id)).await;
        return Ok((removed == 1).then_some(registration));
    }

    if attempt == MAX_OTP_ATTEMPTS {
        let _: Result<(), _> = redis
            .expire(attempts_key(email, id), OTP_LOCK_SECONDS)
            .await;
        return Err(AppError::RateLimited {
            retry_after: OTP_LOCK_SECONDS as u64,
        });
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_otp_consistency() {
        let otp = "123456";
        let hash1 = hash_otp(otp);
        let hash2 = hash_otp(otp);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn registrations_of_one_address_never_share_a_key() {
        let a = registration_key("x@example.com", Uuid::new_v4());
        let b = registration_key("x@example.com", Uuid::new_v4());
        assert_ne!(a, b);
        assert!(a.starts_with("otp:x@example.com:"));
    }
}
