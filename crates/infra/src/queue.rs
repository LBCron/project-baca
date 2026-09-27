//! Redis Streams ingestion queue and per-job status hashes.
//!
//! Uploads publish one message per EPUB; the Python worker consumes via the
//! `ingestion-workers` group. Message fields: `book_id`, `storage_path`,
//! `job_id`, `timestamp`. Job hashes expire after 7 days.

use redis::AsyncCommands;
use shared::{AppError, JobStatusDto};
use std::collections::HashMap;
use uuid::Uuid;

/// Canonical ingestion stream name (single source of truth; docs mirror this).
pub const INGESTION_STREAM: &str = "stream:epub_ingestion";
/// Consumer group used by ingestion workers (created with MKSTREAM on first run).
pub const INGESTION_GROUP: &str = "ingestion-workers";
/// Redis key prefix for per-job status hashes.
pub const JOB_KEY_PREFIX: &str = "job:";
/// Job hash time-to-live in seconds (7 days).
pub const JOB_TTL_SECS: u64 = 7 * 24 * 3600;

fn job_key(job_id: &str) -> String {
    format!("{JOB_KEY_PREFIX}{job_id}")
}

/// Publishes an EPUB ingestion job: appends to the stream and initializes the
/// job-status hash. Returns the new `job_id`.
pub async fn publish_ingestion_job(
    redis: &redis::Client,
    book_id: Uuid,
    storage_path: &str,
) -> Result<String, AppError> {
    let job_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let mut conn = redis
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {e}")))?;

    let _: String = conn
        .xadd(
            INGESTION_STREAM,
            "*",
            &[
                ("book_id", book_id.to_string()),
                ("storage_path", storage_path.to_string()),
                ("job_id", job_id.clone()),
                ("timestamp", now.clone()),
            ],
        )
        .await
        .map_err(|e| AppError::Internal(format!("Stream publish failed: {e}")))?;

    let _: () = conn
        .hset_multiple(
            job_key(&job_id),
            &[
                ("status", "queued"),
                ("progress", "0"),
                ("book_id", &book_id.to_string()),
                ("storage_path", storage_path),
                ("created_at", &now),
                ("updated_at", &now),
            ],
        )
        .await
        .map_err(|e| AppError::Internal(format!("Job hash init failed: {e}")))?;

    let _: () = conn
        .expire(job_key(&job_id), JOB_TTL_SECS as i64)
        .await
        .map_err(|e| AppError::Internal(format!("Job TTL set failed: {e}")))?;

    Ok(job_id)
}

/// Reads a job-status hash. Returns `None` for unknown or expired jobs.
pub async fn get_job_status(
    redis: &redis::Client,
    job_id: &str,
) -> Result<Option<JobStatusDto>, AppError> {
    let mut conn = redis
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {e}")))?;

    let fields: HashMap<String, String> = conn
        .hgetall(job_key(job_id))
        .await
        .map_err(|e| AppError::Internal(format!("Job lookup failed: {e}")))?;

    if fields.is_empty() {
        return Ok(None);
    }

    let book_id = fields
        .get("book_id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::Internal("Job hash missing book_id".to_string()))?;
    let progress = fields
        .get("progress")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    Ok(Some(JobStatusDto {
        job_id: job_id.to_string(),
        book_id,
        status: fields.get("status").cloned().unwrap_or_default(),
        progress,
    }))
}
