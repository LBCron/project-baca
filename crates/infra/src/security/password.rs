//! Argon2id password hashing and verification complying with OWASP guidelines.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};
use shared::AppError;

/// Pre-calculated Argon2id dummy hash for timing attack mitigation (OWASP ASVS V2.1.1).
/// Used during failed authentication when the user email does not exist.
pub const DUMMY_ARGON2_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$dW5rbm93bnVzZXJzYWx0$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

/// Hash a plaintext password using Argon2id with OWASP-recommended parameters:
/// 19MB memory (19456 KiB), 2 iterations, 1 parallelism lane.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| AppError::Internal(format!("Invalid Argon2 parameters: {e}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

/// Verify a plaintext password against an Argon2id hash string
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return Ok(false),
    };

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Offloads Argon2id password hashing to the Tokio blocking thread pool,
/// preventing async executor thread starvation and latency spikes.
pub async fn hash_password_async(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("Password hashing task panicked: {e}")))?
}

/// Offloads Argon2id password verification to the Tokio blocking thread pool.
pub async fn verify_password_async(
    password: String,
    password_hash: String,
) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || verify_password(&password, &password_hash))
        .await
        .map_err(|e| AppError::Internal(format!("Password verification task panicked: {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password_success() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password).expect("Hashing should succeed");

        assert!(hash.starts_with("$argon2id$"));
        let valid = verify_password(password, &hash).expect("Verification should succeed");
        assert!(valid);
    }

    #[test]
    fn test_verify_password_wrong_password() {
        let password = "CorrectPassword123!";
        let hash = hash_password(password).expect("Hashing should succeed");

        let valid = verify_password("WrongPassword!", &hash).expect("Verification should succeed");
        assert!(!valid);
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let valid = verify_password("Password", "invalid_hash_string").expect("Should not panic");
        assert!(!valid);
    }

    #[test]
    fn test_dummy_argon2_hash_validity() {
        let valid =
            verify_password("AnyPassword!", DUMMY_ARGON2_HASH).expect("Should parse dummy hash");
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_async_hash_and_verify() {
        let password = "AsyncSecretPassword123!".to_string();
        let hash = hash_password_async(password.clone())
            .await
            .expect("Async hash should succeed");
        let valid = verify_password_async(password, hash)
            .await
            .expect("Async verify should succeed");
        assert!(valid);
    }
}
