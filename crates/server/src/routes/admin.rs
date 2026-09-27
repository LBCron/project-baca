//! Admin EPUB ingestion endpoints.
use crate::{
    error::HttpError,
    middleware::{require_admin, AuthUser},
    AppState,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use infra::{
    chapter_dropoff, entities::books, get_book_by_id, get_job_status, publish_ingestion_job,
};
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use serde::Deserialize;
use shared::{ApiResponse, AppError, DropOffPointDto, JobStatusDto, UploadBookResponseDto};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub const MAX_EPUB_BYTES: usize = 50 * 1024 * 1024;

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/books/upload", post(upload_book))
        .route("/jobs/{job_id}", get(ingestion_status))
        .route("/analytics/drop-off", get(dropoff_analytics))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/books/upload",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 202, description = "EPUB accepted; ingestion queued", body = UploadBookResponseDto),
        (status = 400, description = "Invalid file type, empty file, or over 50 MB"),
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
    require_admin(&auth_user).map_err(HttpError)?;

    let storage = state.storage.clone().ok_or_else(|| {
        HttpError(AppError::Internal(
            "Object storage unavailable; upload rejected".to_string(),
        ))
    })?;

    let mut file_bytes: Option<(String, Vec<u8>)> = None;
    let mut title: Option<String> = None;
    let mut author: Option<String> = None;
    let mut language = String::from("en");
    let mut theme = String::from("Uncategorized");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| HttpError(AppError::BadRequest(format!("Multipart read failed: {e}"))))?
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
                .map_err(|e| HttpError(AppError::BadRequest(format!("Upload read failed: {e}"))))?
            {
                if buf.len() + chunk.len() > MAX_EPUB_BYTES {
                    return Err(HttpError(AppError::BadRequest(
                        "EPUB exceeds the 50 MB upload limit".to_string(),
                    )));
                }
                buf.extend_from_slice(&chunk);
            }
            if buf.is_empty() {
                return Err(HttpError(AppError::BadRequest(
                    "Uploaded EPUB file is empty".to_string(),
                )));
            }
            file_bytes = Some((filename, buf));
        } else {
            let text = field
                .text()
                .await
                .map_err(|e| HttpError(AppError::BadRequest(format!("Field read failed: {e}"))))?;
            match name.as_str() {
                "title" => title = Some(text),
                "author" => author = Some(text),
                "language" if !text.trim().is_empty() => language = text,
                "theme" if !text.trim().is_empty() => theme = text,
                _ => {}
            }
        }
    }

    let (filename, bytes) = file_bytes.ok_or_else(|| {
        HttpError(AppError::BadRequest(
            "Missing multipart 'file' field with the EPUB payload".to_string(),
        ))
    })?;

    // Reject overlong metadata before the database does (VARCHAR limits).
    let check_len = |name: &str, value: &str, max: usize| {
        if value.len() > max {
            return Err(HttpError(AppError::BadRequest(format!(
                "Field '{name}' exceeds {max} characters"
            ))));
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
    let book_title = title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| filename.trim_end_matches(".epub").replace(['_', '-'], " "));
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
    book.insert(&state.db)
        .await
        .map_err(|e| HttpError(AppError::Database(format!("Failed to record book: {e}"))))?;

    let job_id = publish_ingestion_job(&state.redis, book_id, &storage_path)
        .await
        .map_err(HttpError)?;

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
    require_admin(&auth_user).map_err(HttpError)?;

    let status = get_job_status(&state.redis, &job_id)
        .await
        .map_err(HttpError)?
        .ok_or_else(|| {
            HttpError(AppError::NotFound(format!(
                "Ingestion job not found: {job_id}"
            )))
        })?;

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
    require_admin(&auth_user).map_err(HttpError)?;

    get_book_by_id(&state.db, query.book_id)
        .await
        .map_err(HttpError)?;
    let funnel = chapter_dropoff(&state.db, query.book_id)
        .await
        .map_err(HttpError)?;

    Ok((StatusCode::OK, Json(ApiResponse::success(funnel))))
}
