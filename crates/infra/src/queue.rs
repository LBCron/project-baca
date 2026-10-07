//! Redis Streams ingestion queue and per-job status hashes.
//!
//! Message fields: `book_id`, `storage_path`, `job_id`, `timestamp`.
//! Job hashes expire after 7 days.

use redis::AsyncCommands;
use shared::{AppError, DlqEntryDto, JobStatusDto};
use std::collections::HashMap;
use uuid::Uuid;

/// Canonical ingestion stream name (single source of truth; docs mirror this).
pub const INGESTION_STREAM: &str = "stream:epub_ingestion";
/// Dead-letter stream the worker writes after the last failed attempt.
pub const INGESTION_DLQ_STREAM: &str = "stream:epub_ingestion:dlq";
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

fn stream_entry_time(entry_id: &str) -> chrono::DateTime<chrono::Utc> {
    entry_id
        .split('-')
        .next()
        .and_then(|ms| ms.parse::<i64>().ok())
        .and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_millis)
        .unwrap_or_default()
}

fn dlq_entry(id: String, fields: &HashMap<String, String>) -> DlqEntryDto {
    DlqEntryDto {
        failed_at: stream_entry_time(&id),
        id,
        job_id: fields.get("job_id").cloned().unwrap_or_default(),
        error: fields.get("error").cloned().unwrap_or_default(),
        book_id: fields.get("book_id").and_then(|s| Uuid::parse_str(s).ok()),
    }
}

/// Newest dead-letter entries first.
pub async fn list_dlq_entries(
    redis: &redis::Client,
    limit: usize,
) -> Result<Vec<DlqEntryDto>, AppError> {
    let mut conn = redis
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {e}")))?;
    let reply: redis::streams::StreamRangeReply = conn
        .xrevrange_count(INGESTION_DLQ_STREAM, "+", "-", limit.clamp(1, 200))
        .await
        .map_err(|e| AppError::Internal(format!("DLQ read failed: {e}")))?;
    Ok(reply
        .ids
        .into_iter()
        .map(|entry| {
            let fields: HashMap<String, String> = entry
                .map
                .iter()
                .filter_map(|(k, v)| {
                    redis::from_redis_value::<String>(v)
                        .ok()
                        .map(|s| (k.clone(), s))
                })
                .collect();
            dlq_entry(entry.id, &fields)
        })
        .collect())
}

/// Re-queues one dead-letter entry under its original job id and removes it
/// from the DLQ. `Ok(None)` when the entry is already gone, so a second replay
/// is a no-op; an expired job hash is an error because the storage path is lost.
pub async fn replay_dlq_entry(
    redis: &redis::Client,
    entry_id: &str,
) -> Result<Option<String>, AppError> {
    let mut conn = redis
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {e}")))?;
    let reply: redis::streams::StreamRangeReply = conn
        .xrange(INGESTION_DLQ_STREAM, entry_id, entry_id)
        .await
        .map_err(|e| AppError::Internal(format!("DLQ read failed: {e}")))?;
    let Some(entry) = reply.ids.into_iter().next() else {
        return Ok(None);
    };
    let job_id: String = entry
        .map
        .get("job_id")
        .and_then(|v| redis::from_redis_value::<String>(v).ok())
        .ok_or_else(|| AppError::Internal("DLQ entry without job_id".to_string()))?;

    let job: HashMap<String, String> = conn
        .hgetall(job_key(&job_id))
        .await
        .map_err(|e| AppError::Internal(format!("Job lookup failed: {e}")))?;
    let (Some(book_id), Some(storage_path)) = (job.get("book_id"), job.get("storage_path")) else {
        return Err(AppError::Conflict(
            "Job record expired; upload the EPUB again".to_string(),
        ));
    };

    let now = chrono::Utc::now().to_rfc3339();
    let _: String = conn
        .xadd(
            INGESTION_STREAM,
            "*",
            &[
                ("book_id", book_id.as_str()),
                ("storage_path", storage_path.as_str()),
                ("job_id", job_id.as_str()),
                ("timestamp", now.as_str()),
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
                ("attempts", "0"),
                ("error", ""),
                ("updated_at", now.as_str()),
            ],
        )
        .await
        .map_err(|e| AppError::Internal(format!("Job hash reset failed: {e}")))?;
    let _: () = conn
        .expire(job_key(&job_id), JOB_TTL_SECS as i64)
        .await
        .map_err(|e| AppError::Internal(format!("Job TTL set failed: {e}")))?;
    let _: () = conn
        .xdel(INGESTION_DLQ_STREAM, &[entry_id])
        .await
        .map_err(|e| AppError::Internal(format!("DLQ acknowledge failed: {e}")))?;
    Ok(Some(job_id))
}
