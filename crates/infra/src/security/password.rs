//! Argon2id password hashing and verification complying with OWASP guidelines.

use argon2::{Argon2, Params, PasswordHasher, PasswordVerifier};
use shared::AppError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// How long a request waits for an Argon2 slot before it is refused.
const ARGON2_WAIT: Duration = Duration::from_secs(2);

/// Pre-calculated Argon2id dummy hash for timing attack mitigation (OWASP ASVS V2.1.1).
/// Used during failed authentication when the user email does not exist.
pub const DUMMY_ARGON2_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$dW5rbm93bnVzZXJzYWx0$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

/// Hash a plaintext password using Argon2id with OWASP-recommended parameters:
/// 19MB memory (19456 KiB), 2 iterations, 1 parallelism lane.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| AppError::Internal(format!("Invalid Argon2 parameters: {e}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    // 16-byte salt from the OS (argon2's default `getrandom` feature).
    argon2
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

/// Verify a plaintext password against an Argon2id hash string
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    // Cost parameters come from the PHC string; an unparsable one is a miss.
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), password_hash)
        .is_ok())
}

/// Bounds how many Argon2 operations run at once. Each one holds 19 MiB and
/// the blocking pool would run hundreds side by side under a login flood;
/// past the gate a caller waits a little, then gets 503 instead of the
/// process meeting the OOM killer.
#[derive(Clone)]
pub struct Argon2Gate {
    permits: Arc<Semaphore>,
    wait: Duration,
}

impl Argon2Gate {
    pub fn new(max_concurrency: usize) -> Self {
        Self::with_wait(max_concurrency, ARGON2_WAIT)
    }

    fn with_wait(max_concurrency: usize, wait: Duration) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(max_concurrency.max(1))),
            wait,
        }
    }

    /// One Argon2 slot, or 503 when none frees up within the wait.
    pub async fn permit(&self) -> Result<OwnedSemaphorePermit, AppError> {
        tokio::time::timeout(self.wait, Arc::clone(&self.permits).acquire_owned())
            .await
            .map_err(|_| AppError::ServiceUnavailable { retry_after: 1 })?
            .map_err(|_| AppError::Internal("Argon2 gate closed".to_string()))
    }

    /// Runs `work` on the blocking pool inside a slot. The slot moves into
    /// the blocking task, so a request dropped mid-hash still holds it until
    /// the hash ends and the bound covers the memory actually in use.
    async fn run<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> Result<T, AppError> + Send + 'static,
    ) -> Result<T, AppError> {
        let permit = self.permit().await?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            work()
        })
        .await
        .map_err(|e| AppError::Internal(format!("Argon2 task panicked: {e}")))?
    }

    pub async fn hash(&self, password: String) -> Result<String, AppError> {
        self.run(move || hash_password(&password)).await
    }

    pub async fn verify(&self, password: String, password_hash: String) -> Result<bool, AppError> {
        self.run(move || verify_password(&password, &password_hash))
            .await
    }
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

    /// Hash written by argon2 0.5 before the crate upgrade; stored hashes
    /// must keep verifying across argon2 releases.
    const LEGACY_PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$+rRVNarSMsOtcCUyWVMhzg$gtawrOhAO0n1vrR8hxub3Ybi/oZ0jLF6JgXYf835fAo";

    #[test]
    fn hashes_from_earlier_releases_still_verify() {
        assert!(verify_password("Rotaria-lama-0.5!", LEGACY_PHC).expect("legacy hash parses"));
        assert!(!verify_password("Rotaria-lama-0.5?", LEGACY_PHC).expect("legacy hash parses"));
    }

    #[test]
    fn new_hashes_keep_parameters_and_salt_length() {
        let hash = hash_password("Whatever123!").expect("Hashing should succeed");
        let salt_and_digest = hash
            .strip_prefix("$argon2id$v=19$m=19456,t=2,p=1$")
            .expect("same algorithm, version and cost");
        let (salt, digest) = salt_and_digest.split_once('$').expect("salt then digest");
        assert_eq!(salt.len(), 22, "16-byte salt, unpadded base64");
        assert_eq!(digest.len(), 43, "32-byte output, unpadded base64");
    }

    #[test]
    fn test_dummy_argon2_hash_validity() {
        let valid =
            verify_password("AnyPassword!", DUMMY_ARGON2_HASH).expect("Should parse dummy hash");
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_async_hash_and_verify() {
        let gate = Argon2Gate::new(2);
        let password = "AsyncSecretPassword123!".to_string();
        let hash = gate
            .hash(password.clone())
            .await
            .expect("Async hash should succeed");
        let valid = gate
            .verify(password, hash)
            .await
            .expect("Async verify should succeed");
        assert!(valid);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn gate_never_runs_more_than_its_permits() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let gate = Arc::new(Argon2Gate::new(2));
        let running = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let calls: Vec<_> = (0..20)
            .map(|_| {
                let (gate, running, peak) = (gate.clone(), running.clone(), peak.clone());
                tokio::spawn(async move {
                    gate.run(move || {
                        let now = running.fetch_add(1, Ordering::SeqCst) + 1;
                        peak.fetch_max(now, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(20));
                        running.fetch_sub(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .await
                })
            })
            .collect();
        for call in calls {
            call.await
                .expect("task must not panic")
                .expect("slot must free up in time");
        }
        assert_eq!(peak.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_caller_without_a_slot_in_time_gets_503() {
        let gate = Argon2Gate::with_wait(1, Duration::from_millis(50));
        let _held = gate.permit().await.expect("first slot is free");
        let refused = gate.hash("Whatever123!".to_string()).await;
        assert!(matches!(
            refused,
            Err(AppError::ServiceUnavailable { retry_after: 1 })
        ));
    }
}
