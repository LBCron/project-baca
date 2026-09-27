//! Reading progress and guest reconciliation endpoints.

use crate::{error::HttpError, middleware::auth::AuthUser, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use sea_orm::{entity::prelude::Decimal, ConnectionTrait, DatabaseBackend, Statement};
use shared::{
    ActiveProgressDto, ApiResponse, ErrorPayload, GuestMergeRequest, UpdateProgressRequest,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Latest unfinished reading position for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/progress/active",
    responses(
        (status = 200, description = "Active reading progress or null if none", body = ApiResponse<Option<ActiveProgressDto>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "Reading Progress"
)]
pub async fn get_active_progress(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Response, HttpError> {
    let progress = infra::get_active_progress(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(progress)).into_response())
}

/// Persists reading position (chapter, CFI anchor, percentage).
#[utoipa::path(
    put,
    path = "/api/v1/progress/{book_id}",
    params(
        ("book_id" = Uuid, Path, description = "Book UUID")
    ),
    request_body = UpdateProgressRequest,
    responses(
        (status = 200, description = "Reading progress updated successfully", body = ApiResponse<String>),
        (status = 400, description = "Validation failed", body = ApiResponse<ErrorPayload>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>),
        (status = 404, description = "Book or chapter not found", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "Reading Progress"
)]
pub async fn update_progress(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(book_id): Path<Uuid>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    infra::update_progress(&state.db, auth.id, book_id, &req)
        .await
        .map_err(HttpError::from)?;

    Ok(Json(ApiResponse::success(
        "Reading progress updated successfully".to_string(),
    ))
    .into_response())
}

/// Merges guest progress into the account, keeping the highest completion.
#[utoipa::path(
    post,
    path = "/api/v1/progress/merge",
    request_body = GuestMergeRequest,
    responses(
        (status = 200, description = "Guest progress reconciled successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid payload", body = ApiResponse<ErrorPayload>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "Reading Progress"
)]
pub async fn merge_guest_progress(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<GuestMergeRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut merged_count = 0;

    for record in req.records {
        let decimal_str = format!("{:.2}", record.completion_percentage);
        let completion: Decimal = decimal_str.parse().unwrap_or(Decimal::ZERO);
        let last_read_at = record.last_read_at.unwrap_or_else(Utc::now);
        let is_finished = record.is_finished.unwrap_or(false);

        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
            INSERT INTO user_reading_progress (
                id, user_id, book_id, last_chapter_id, last_anchor_cfi,
                completion_percentage, is_finished, last_read_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            ON CONFLICT (user_id, book_id) DO UPDATE SET
                last_chapter_id = EXCLUDED.last_chapter_id,
                last_anchor_cfi = EXCLUDED.last_anchor_cfi,
                completion_percentage = GREATEST(user_reading_progress.completion_percentage, EXCLUDED.completion_percentage),
                is_finished = (user_reading_progress.is_finished OR EXCLUDED.is_finished),
                last_read_at = GREATEST(user_reading_progress.last_read_at, EXCLUDED.last_read_at),
                updated_at = NOW();
            "#,
            vec![
                Uuid::new_v4().into(),
                auth.id.into(),
                record.book_id.into(),
                record.last_chapter_id.into(),
                record.last_anchor_cfi.into(),
                completion.into(),
                is_finished.into(),
                last_read_at.into(),
            ],
        );

        state.db.execute(stmt).await.map_err(HttpError::from)?;
        merged_count += 1;
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Guest progress successfully merged",
            "merged_count": merged_count
        }))),
    )
        .into_response())
}

pub fn progress_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/active", get(get_active_progress))
        .route("/{book_id}", put(update_progress))
        .route("/merge", post(merge_guest_progress))
}
