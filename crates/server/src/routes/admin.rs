//! Curator endpoints: EPUB ingestion, the catalog in every status, the
//! lifecycle guard, the dead-letter queue, and the drop-off funnel.
use crate::{
    error::HttpError,
    middleware::{require_admin, AuthUser},
    AppState,
};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use domain::BookStatus;
use infra::{
    book_titles, chapter_dropoff, dismiss_dlq_entry, dlq_length, entities::books, get_book_by_id,
    get_book_for_admin, get_book_status, get_job_status, is_fully_ingested, list_books_for_admin,
    list_dlq_entries, publish_ingestion_job, replay_dlq_entry, set_book_status,
};
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use serde::Deserialize;
use shared::{
    AdminBookPatchRequest, AdminBookRowDto, ApiResponse, AppError, DlqPageDto,
    DlqReplayResponseDto, DropOffPointDto, JobStatusDto, UploadBookResponseDto, MAX_EPUB_BYTES,
};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

const TOO_LARGE: &str = "EPUB exceeds the 50 MB upload limit";

/// A multipart read error, as 413 when the body limit cut it short (axum
/// would otherwise report it as a malformed form).
fn multipart_error(context: &str, e: axum::extract::multipart::MultipartError) -> HttpError {
    if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return HttpError(AppError::PayloadTooLarge(TOO_LARGE.to_string()));
    }
    HttpError(AppError::BadRequest(format!("{context}: {e}")))
}

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/books", get(list_catalog))
        // Multipart EPUB uploads need more than axum's 2 MB default; the
        // handler still enforces the 50 MB product limit while streaming.
        // Every other admin route keeps the default.
        .route(
            "/books/upload",
            post(upload_book).layer(DefaultBodyLimit::max(MAX_EPUB_BYTES + 64 * 1024)),
        )
        .route("/books/{book_id}", patch(patch_book))
        .route("/jobs/{job_id}", get(ingestion_status))
        .route("/dlq", get(list_dlq))
        .route("/dlq/{entry_id}", delete(dismiss_dlq))
        .route("/dlq/{entry_id}/replay", post(replay_dlq))
        .route("/analytics/drop-off", get(dropoff_analytics))
}

/// Filters for the curator catalog.
#[derive(Debug, Deserialize)]
pub struct AdminCatalogQuery {
    status: Option<String>,
    cursor: Option<Uuid>,
    limit: Option<u64>,
}

/// Every manuscript in every status, newest first, with content counts.
#[utoipa::path(
    get,
    path = "/api/v1/admin/books",
    params(
        ("status" = Option<String>, Query, description = "draft, processing, published, or archived"),
        ("cursor" = Option<Uuid>, Query, description = "Last book id of the previous page"),
        ("limit" = Option<u64>, Query, description = "Page size, 1-100 (default 50)")
    ),
    responses(
        (status = 200, description = "Catalog rows, newest first", body = Vec<AdminBookRowDto>),
        (status = 400, description = "Unknown status filter"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn list_catalog(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(query): Query<AdminCatalogQuery>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;
    if let Some(status) = &query.status {
        BookStatus::from_str(status).map_err(|e| HttpError(e.into()))?;
    }
    let rows = list_books_for_admin(
        &state.db,
        query.status.as_deref(),
        query.cursor,
        query.limit.unwrap_or(50),
    )
    .await
    .map_err(HttpError)?;
    Ok((StatusCode::OK, Json(ApiResponse::success(rows))))
}

/// Moves a manuscript along its lifecycle; the domain refuses illegal steps
/// (a draft cannot be published by hand), and an archived book is published
/// again only if its ingestion had finished.
#[utoipa::path(
    patch,
    path = "/api/v1/admin/books/{book_id}",
    params(("book_id" = Uuid, Path, description = "Book to update")),
    request_body = AdminBookPatchRequest,
    responses(
        (status = 200, description = "New status applied", body = AdminBookRowDto),
        (status = 400, description = "Unknown status"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Book not found"),
        (status = 409, description = "Transition not allowed by the lifecycle")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn patch_book(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(book_id): Path<Uuid>,
    Json(req): Json<AdminBookPatchRequest>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;
    req.validate().map_err(HttpError::from)?;
    let next = BookStatus::from_str(&req.status).map_err(|e| HttpError(e.into()))?;
    let current_raw = get_book_status(&state.db, book_id)
        .await
        .map_err(HttpError)?;
    let current = BookStatus::from_str(&current_raw).map_err(|e| HttpError(e.into()))?;
    if !current.can_transition_to(next) {
        return Err(HttpError(AppError::Conflict(format!(
            "A book that is {current_raw} cannot become {}",
            req.status
        ))));
    }
    if current == BookStatus::Archived
        && next == BookStatus::Published
        && !is_fully_ingested(&state.db, book_id)
            .await
            .map_err(HttpError)?
    {
        return Err(HttpError(AppError::Conflict(
            "This book was archived before its ingestion finished; upload it again".to_string(),
        )));
    }
    if current != next {
        set_book_status(&state.db, book_id, &req.status)
            .await
            .map_err(HttpError)?;
    }
    let row = get_book_for_admin(&state.db, book_id)
        .await
        .map_err(HttpError)?
        .ok_or_else(|| HttpError(AppError::NotFound("Book not found".to_string())))?;
    Ok((StatusCode::OK, Json(ApiResponse::success(row))))
}

/// Page size for the dead-letter listing.
#[derive(Debug, Deserialize)]
pub struct DlqQuery {
    limit: Option<usize>,
}

/// Dead-letter entries, newest first, with the book title and the stream total.
#[utoipa::path(
    get,
    path = "/api/v1/admin/dlq",
    params(("limit" = Option<usize>, Query, description = "Entries to return, 1-200 (default 50)")),
    responses(
        (status = 200, description = "Dead-letter page", body = DlqPageDto),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn list_dlq(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(query): Query<DlqQuery>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;
    let mut redis = state.get_redis_conn().await.map_err(HttpError)?;
    let mut entries = list_dlq_entries(&mut redis, query.limit.unwrap_or(50))
        .await
        .map_err(HttpError)?;
    let total = dlq_length(&mut redis).await.map_err(HttpError)?;
    let titles = book_titles(
        &state.db,
        entries.iter().filter_map(|e| e.book_id).collect(),
    )
    .await
    .map_err(HttpError)?;
    for entry in &mut entries {
        entry.book_title = entry.book_id.and_then(|id| titles.get(&id).cloned());
    }
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(DlqPageDto { total, entries })),
    ))
}

/// Removes one dead letter for good, for a job that should not run again.
#[utoipa::path(
    delete,
    path = "/api/v1/admin/dlq/{entry_id}",
    params(("entry_id" = String, Path, description = "Dead-letter stream entry id")),
    responses(
        (status = 200, description = "Entry removed: `{\"dismissed\": true}`"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Entry not found (already removed or replayed)")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn dismiss_dlq(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(entry_id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;
    let mut redis = state.get_redis_conn().await.map_err(HttpError)?;
    if !dismiss_dlq_entry(&mut redis, &entry_id)
        .await
        .map_err(HttpError)?
    {
        return Err(HttpError(AppError::NotFound(
            "Dead-letter entry not found".to_string(),
        )));
    }
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            serde_json::json!({ "dismissed": true }),
        )),
    ))
}

/// Re-queues one dead-letter entry under its original job id. A second call
/// for the same entry finds nothing and answers 404, so replays never double.
#[utoipa::path(
    post,
    path = "/api/v1/admin/dlq/{entry_id}/replay",
    params(("entry_id" = String, Path, description = "Dead-letter stream entry id")),
    responses(
        (status = 200, description = "Job re-queued", body = DlqReplayResponseDto),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Entry not found (already replayed or removed)"),
        (status = 409, description = "Job record expired; the EPUB must be uploaded again")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn replay_dlq(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(entry_id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;
    let mut redis = state.get_redis_conn().await.map_err(HttpError)?;
    let job_id = replay_dlq_entry(&mut redis, &entry_id)
        .await
        .map_err(HttpError)?
        .ok_or_else(|| {
            HttpError(AppError::NotFound(
                "Dead-letter entry not found".to_string(),
            ))
        })?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(DlqReplayResponseDto {
            job_id,
            replayed: true,
        })),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/books/upload",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 202, description = "EPUB accepted; ingestion queued", body = UploadBookResponseDto),
        (status = 400, description = "Invalid file type or empty file"),
        (status = 413, description = "Over 50 MB"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required"),
        (status = 500, description = "Storage, database, or queue error")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
/// Accepts an EPUB via multipart, stores it, and queues ingestion. 202 on accept.
pub async fn upload_book(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;

    // A caller-controlled job id would let anyone read arbitrary job hashes,
    // so validate the path parameter strictly on the read side.
    let storage = state.storage.clone().ok_or_else(|| {
        HttpError(AppError::Internal(
            "Object storage unavailable; upload rejected".to_string(),
        ))
    })?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut title: Option<String> = None;
    let mut author: Option<String> = None;
    // Left blank, title and language are read from the EPUB by the worker.
    let mut language = String::new();
    let mut theme = String::from("Uncategorized");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| multipart_error("Multipart read failed", e))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or_default().to_string();
            let content_type = field.content_type().unwrap_or_default().to_string();
            let is_epub = content_type == "application/epub+zip"
                || filename.to_lowercase().ends_with(".epub");
            if !is_epub {
                return Err(HttpError(AppError::BadRequest(format!(
                    "Only EPUB files are accepted (got content-type '{content_type}', filename '{filename}')"
                ))));
            }
            let mut buf = Vec::new();
            let mut field_stream = field;
            while let Some(chunk) = field_stream
                .chunk()
                .await
                .map_err(|e| multipart_error("Upload read failed", e))?
            {
                if buf.len() + chunk.len() > MAX_EPUB_BYTES {
                    return Err(HttpError(AppError::PayloadTooLarge(TOO_LARGE.to_string())));
                }
                buf.extend_from_slice(&chunk);
            }
            if buf.is_empty() {
                return Err(HttpError(AppError::BadRequest(
                    "Uploaded EPUB file is empty".to_string(),
                )));
            }
            if buf.len() < 2 || buf[0] != 0x50 || buf[1] != 0x4B {
                return Err(HttpError(AppError::BadRequest(
                    "Uploaded file is not a ZIP-based EPUB (missing PK signature)".to_string(),
                )));
            }
            file_bytes = Some(buf);
        } else {
            // Text fields arrive buffered by axum; cap them before the DB does
            // so one giant field cannot exhaust server memory.
            const MAX_TEXT_FIELD_BYTES: usize = 8 * 1024;
            let text = field
                .text()
                .await
                .map_err(|e| multipart_error("Field read failed", e))?;
            if text.len() > MAX_TEXT_FIELD_BYTES {
                return Err(HttpError(AppError::BadRequest(format!(
                    "Field '{name}' exceeds 8 KiB"
                ))));
            }
            match name.as_str() {
                "title" => title = Some(text),
                "author" => author = Some(text),
                "language" if !text.trim().is_empty() => language = text,
                "theme" if !text.trim().is_empty() => theme = text,
                _ => {}
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        HttpError(AppError::BadRequest(
            "Missing multipart 'file' field with the EPUB payload".to_string(),
        ))
    })?;

    // Reject overlong metadata before the database does. VARCHAR(n) counts
    // characters, as the form's maxlength roughly does, so bytes would refuse
    // an accented title the form let through. Named per field, so the curator
    // is told which one to shorten.
    let check_len = |name: &str, value: &str, max: usize| {
        if value.chars().count() > max {
            let message = format!("Field '{name}' exceeds {max} characters");
            return Err(HttpError(AppError::ValidationDetailed {
                details: serde_json::json!({ name: [message.clone()] }),
                message,
            }));
        }
        Ok(())
    };
    if let Some(t) = &title {
        check_len("title", t, 255)?;
    }
    if let Some(a) = &author {
        check_len("author", a, 255)?;
    }
    check_len("language", &language, 10)?;
    check_len("theme", &theme, 50)?;

    let book_id = Uuid::new_v4();
    let storage_path = format!("raw-epubs/{book_id}.epub");
    storage
        .put_epub(&storage_path, bytes, "application/epub+zip")
        .await
        .map_err(HttpError)?;

    let now = chrono::Utc::now();
    let book_title = title.map(|t| t.trim().to_string()).unwrap_or_default();
    let book = books::ActiveModel {
        id: Set(book_id),
        title: Set(book_title),
        author: Set(author
            .filter(|a| !a.trim().is_empty())
            .unwrap_or_else(|| "Unknown".to_string())),
        language: Set(language),
        primary_theme: Set(theme),
        sub_theme: Set(None),
        description: Set(String::new()),
        cover_url: Set(String::new()),
        epub_storage_path: Set(storage_path.clone()),
        total_words: Set(0),
        estimated_reading_minutes: Set(0),
        source_name: Set("Admin Upload".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(None),
        status: Set("draft".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    if let Err(e) = book.insert(&state.db).await {
        // The EPUB bytes are already in S3: remove them so a DB failure does
        // not leave an orphan object with no catalog row.
        if let Err(del_err) = storage.delete_epub(&storage_path).await {
            tracing::warn!(target: "server::admin", "Orphan EPUB cleanup failed: {del_err}");
        }
        return Err(HttpError(AppError::Database(format!(
            "Failed to record book: {e}"
        ))));
    }

    let published = match state.get_redis_conn().await {
        Ok(mut redis) => publish_ingestion_job(&mut redis, book_id, &storage_path).await,
        Err(e) => Err(e),
    };
    let job_id = published.map_err(|e| {
        // The book row stays as `draft` (visible to admins, invisible to
        // readers) so a retry or reaper can pick it up later.
        tracing::warn!(target: "server::admin", "Queue publish failed; book left as draft: {e}");
        HttpError::from(e)
    })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiResponse::success(UploadBookResponseDto {
            job_id,
            book_id,
            status: "queued".to_string(),
        })),
    ))
}

/// Ingestion job snapshot from Redis. 404 for unknown or expired ids.
#[utoipa::path(
    get,
    path = "/api/v1/admin/jobs/{job_id}",
    params(
        ("job_id" = String, Path, description = "Ingestion job ID from the 202 upload response")
    ),
    responses(
        (status = 200, description = "Job status snapshot", body = JobStatusDto),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Unknown or expired job ID"),
        (status = 500, description = "Queue read error")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn ingestion_status(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;

    let job_id = job_id.trim();
    if Uuid::parse_str(job_id).is_err() {
        return Err(HttpError(AppError::NotFound(
            "Ingestion job not found".to_string(),
        )));
    }

    let mut redis = state.get_redis_conn().await.map_err(HttpError)?;
    let status = get_job_status(&mut redis, job_id)
        .await
        .map_err(HttpError)?
        .ok_or_else(|| HttpError(AppError::NotFound("Ingestion job not found".to_string())))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(status))))
}

/// Query parameters for the drop-off funnel (book under analysis).
#[derive(Debug, Deserialize, Validate)]
pub struct DropoffQuery {
    book_id: Uuid,
}

/// Chapter drop-off funnel for one book, ordered by chapter number.
#[utoipa::path(
    get,
    path = "/api/v1/admin/analytics/drop-off",
    params(
        ("book_id" = Uuid, Query, description = "Book to analyze")
    ),
    responses(
        (status = 200, description = "Funnel ordered by chapter number", body = Vec<DropOffPointDto>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Book not found"),
        (status = 500, description = "Database error")
    ),
    security(("BearerAuth" = [])),
    tag = "Administration"
)]
pub async fn dropoff_analytics(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(query): Query<DropoffQuery>,
) -> Result<impl IntoResponse, HttpError> {
    require_admin(&state, &auth_user).await.map_err(HttpError)?;

    get_book_by_id(&state.db, query.book_id)
        .await
        .map_err(HttpError)?;
    let funnel = chapter_dropoff(&state.db, query.book_id)
        .await
        .map_err(HttpError)?;

    Ok((StatusCode::OK, Json(ApiResponse::success(funnel))))
}
