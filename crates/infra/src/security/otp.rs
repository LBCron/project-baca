//! 6-digit numeric OTP generation and Redis SHA-256 storage with rate-limiting.

use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::AppError;

const OTP_TTL_SECONDS: u64 = 600; // 10 minutes
const MAX_OTP_ATTEMPTS: u32 = 3;
const OTP_LOCK_SECONDS: i64 = 300; // 5-minute cooldown after exhausting attempts

#[derive(Debug, Serialize, Deserialize)]
struct OtpRecord {
    pub hash: String,
    pub attempts: u32,
    #[serde(default)]
    pub locked_until: Option<i64>,
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

/// Generates a random 6-digit numeric OTP and stores its SHA-256 hash in Redis
pub async fn generate_and_store_otp(
    redis: &mut redis::aio::MultiplexedConnection,
    email: &str,
) -> Result<String, AppError> {
    let otp_str = generate_numeric_otp();
    let hash = hash_otp(&otp_str);

    let record = OtpRecord {
        hash,
        attempts: 0,
        locked_until: None,
    };
    let json_val = serde_json::to_string(&record)
        .map_err(|e| AppError::Internal(format!("Failed to serialize OTP record: {e}")))?;

    let redis_key = format!("otp:{email}");
    let _: () = redis
        .set_ex(&redis_key, json_val, OTP_TTL_SECONDS)
        .await
        .map_err(|e| AppError::Internal(format!("Redis set_ex failed for OTP: {e}")))?;

    Ok(otp_str)
}

/// Verifies a 6-digit OTP against its Redis record, enforcing attempt limits
pub async fn verify_and_consume_otp(
    redis: &mut redis::aio::MultiplexedConnection,
    email: &str,
    otp: &str,
) -> Result<bool, AppError> {
    let redis_key = format!("otp:{email}");
    let record_str: Option<String> = redis
        .get(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Redis get failed for OTP: {e}")))?;

    let record_str = match record_str {
        Some(s) => s,
        None => return Ok(false),
    };

    let mut record: OtpRecord = match serde_json::from_str(&record_str) {
        Ok(r) => r,
        Err(_) => {
            let _: () = redis.del(&redis_key).await.unwrap_or(());
            return Ok(false);
        }
    };

    let now = chrono::Utc::now().timestamp();
    if let Some(locked_until) = record.locked_until {
        if now < locked_until {
            return Err(AppError::ValidationError(
                "Too many incorrect attempts. Please wait a few minutes or request a new OTP."
                    .to_string(),
            ));
        }
    }

    record.attempts += 1;

    let computed_hash = hash_otp(otp);
    if record.hash == computed_hash {
        let _: () = redis.del(&redis_key).await.unwrap_or(());
        return Ok(true);
    }

    if record.attempts >= MAX_OTP_ATTEMPTS {
        record.locked_until = Some(now + OTP_LOCK_SECONDS);
        let locked_json = serde_json::to_string(&record).unwrap_or(record_str);
        let ttl: i64 = redis.ttl(&redis_key).await.unwrap_or(300);
        let remaining_ttl = if ttl > 0 { ttl as u64 } else { 300 };
        let _: () = redis
            .set_ex(&redis_key, locked_json, remaining_ttl)
            .await
            .unwrap_or(());
        return Err(AppError::ValidationError(
            "Maximum OTP verification attempts exceeded. Please wait a few minutes or request a new OTP.".to_string(),
        ));
    }

    // Save incremented attempts with remaining TTL
    let ttl: i64 = redis.ttl(&redis_key).await.unwrap_or(300);
    let remaining_ttl = if ttl > 0 { ttl as u64 } else { 300 };
    let updated_json = serde_json::to_string(&record).unwrap_or(record_str);
    let _: () = redis
        .set_ex(&redis_key, updated_json, remaining_ttl)
        .await
        .unwrap_or(());

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
