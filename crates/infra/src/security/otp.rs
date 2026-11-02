//! 6-digit numeric OTP generation and Redis SHA-256 storage with attempt
//! limiting: three misses lock the code for five minutes, surfaced as a
//! 429 with `Retry-After` so clients can show a countdown.

use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::AppError;

const OTP_TTL_SECONDS: u64 = 600; // 10 minutes
const MAX_OTP_ATTEMPTS: i64 = 3;
const OTP_LOCK_SECONDS: i64 = 300; // 5-minute cooldown after exhausting attempts

#[derive(Debug, Serialize, Deserialize)]
struct OtpRecord {
    pub hash: String,
}

/// Redis key of the pending code for `email`; it lives as long as the code.
pub fn otp_key(email: &str) -> String {
    format!("otp:{email}")
}

fn attempts_key(email: &str) -> String {
    format!("otp:attempts:{email}")
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

/// Generates a random 6-digit numeric OTP and stores its SHA-256 hash in
/// Redis. A new code starts with a clean attempt count.
pub async fn generate_and_store_otp(
    redis: &mut crate::RedisConn,
    email: &str,
) -> Result<String, AppError> {
    let otp_str = generate_numeric_otp();
    let json_val = serde_json::to_string(&OtpRecord {
        hash: hash_otp(&otp_str),
    })
    .map_err(|e| AppError::Internal(format!("Failed to serialize OTP record: {e}")))?;

    let _: () = redis::pipe()
        .atomic()
        .set_ex(otp_key(email), json_val, OTP_TTL_SECONDS)
        .ignore()
        .del(attempts_key(email))
        .ignore()
        .query_async(redis)
        .await
        .map_err(|e| redis_error("Redis write failed for OTP", e))?;

    Ok(otp_str)
}

/// Verifies a 6-digit OTP against its Redis record, enforcing attempt limits.
///
/// The attempt is counted with one atomic `INCR` before the hash is
/// compared, so parallel guesses each take their own slot and anything past
/// the third is refused unseen.
pub async fn verify_and_consume_otp(
    redis: &mut crate::RedisConn,
    email: &str,
    otp: &str,
) -> Result<bool, AppError> {
    let record_str: Option<String> = redis
        .get(otp_key(email))
        .await
        .map_err(|e| redis_error("Redis get failed for OTP", e))?;
    let Some(record_str) = record_str else {
        return Ok(false);
    };
    let Ok(record) = serde_json::from_str::<OtpRecord>(&record_str) else {
        let _: Result<(), _> = redis.del(otp_key(email)).await;
        return Ok(false);
    };

    let (attempt,): (i64,) = redis::pipe()
        .atomic()
        .cmd("SET")
        .arg(attempts_key(email))
        .arg(0)
        .arg("NX")
        .arg("EX")
        .arg(OTP_TTL_SECONDS)
        .ignore()
        .incr(attempts_key(email), 1)
        .query_async(redis)
        .await
        .map_err(|e| redis_error("Redis attempt count failed for OTP", e))?;

    if attempt > MAX_OTP_ATTEMPTS {
        let ttl: i64 = redis
            .ttl(attempts_key(email))
            .await
            .unwrap_or(OTP_LOCK_SECONDS);
        return Err(AppError::RateLimited {
            retry_after: ttl.max(1) as u64,
        });
    }

    if record.hash == hash_otp(otp) {
        let _: Result<(), _> = redis.del(&[otp_key(email), attempts_key(email)]).await;
        return Ok(true);
    }

    if attempt == MAX_OTP_ATTEMPTS {
        let _: Result<(), _> = redis.expire(attempts_key(email), OTP_LOCK_SECONDS).await;
        return Err(AppError::RateLimited {
            retry_after: OTP_LOCK_SECONDS as u64,
        });
    }

    Ok(false)
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
}
