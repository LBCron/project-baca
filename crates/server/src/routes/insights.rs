//! Atomic insight cards and spoiler-free recap endpoints.

use crate::{error::HttpError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use infra::{get_chapter_by_number, get_chapter_recap as query_chapter_recap, get_tldr_cache};
use shared::{ApiResponse, AppError, AtomicCardsDto, ChapterRecapDto};
use std::sync::Arc;
use uuid::Uuid;

async fn resolve_chapter(
    state: &Arc<AppState>,
    book_id: Uuid,
    chapter_ref: &str,
) -> Result<(Uuid, Option<i32>), AppError> {
    if let Ok(num) = chapter_ref.parse::<i32>() {
        let ch = get_chapter_by_number(&state.db, book_id, num).await?;
        Ok((ch.id, Some(num)))
    } else if let Ok(uuid) = Uuid::parse_str(chapter_ref) {
        Ok((uuid, None))
    } else {
        Err(AppError::ValidationError(format!(
            "Invalid chapter reference '{chapter_ref}': must be a chapter number (e.g. 2) or a UUID."
        )))
    }
}

pub fn insights_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/{book_id}/chapters/{chapter_ref}/atomic-cards",
            get(get_atomic_cards),
        )
        .route(
            "/{book_id}/chapters/{chapter_ref}/recap",
            get(get_chapter_recap),
        )
}

// Chapter Atomic Insight Cards

/// Precomputed atomic cards for a chapter (number or UUID). 404 on cache miss.
#[utoipa::path(
    get,
    path = "/api/v1/books/{book_id}/chapters/{chapter_ref}/atomic-cards",
    params(
        ("book_id" = Uuid, Path, description = "UUID of the book"),
        ("chapter_ref" = String, Path, description = "Chapter number (e.g. 1) or chapter UUID")
    ),
    responses(
        (status = 200, description = "Atomic insight cards retrieved from cache", body = AtomicCardsDto),
        (status = 404, description = "Atomic cards not yet generated for this chapter"),
        (status = 500, description = "Database error")
    ),
    tag = "Semantic Search"
)]
pub async fn get_atomic_cards(
    State(state): State<Arc<AppState>>,
    Path((book_id, chapter_ref)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, HttpError> {
    let (chapter_id, _) = resolve_chapter(&state, book_id, &chapter_ref)
        .await
        .map_err(HttpError)?;

    let maybe_cards = get_tldr_cache(&state.db, book_id, chapter_id, "chapter_atomic_cards")
        .await
        .map_err(HttpError)?;

    match maybe_cards {
        Some(cards_json) => {
            let dto = AtomicCardsDto {
                book_id,
                chapter_id,
                cards: cards_json,
            };
            Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
        }
        None => Err(HttpError(AppError::NotFound(format!(
            "Atomic cards for chapter {chapter_ref} have not been generated yet."
        )))),
    }
}

// Spoiler-Free Catch-up Recap

/// Spoiler-free recap of prior chapters. Chapters > 1 only; 404 otherwise.
#[utoipa::path(
    get,
    path = "/api/v1/books/{book_id}/chapters/{chapter_ref}/recap",
    params(
        ("book_id" = Uuid, Path, description = "UUID of the book"),
        ("chapter_ref" = String, Path, description = "Chapter number (e.g. 2) or chapter UUID")
    ),
    responses(
        (status = 200, description = "Spoiler-free catch-up recap retrieved from cache", body = ChapterRecapDto),
        (status = 404, description = "Recap not yet generated or not applicable for chapter 1"),
        (status = 500, description = "Database error")
    ),
    tag = "Semantic Search"
)]
pub async fn get_chapter_recap(
    State(state): State<Arc<AppState>>,
    Path((book_id, chapter_ref)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, HttpError> {
    let (chapter_id, maybe_num) = resolve_chapter(&state, book_id, &chapter_ref)
        .await
        .map_err(HttpError)?;

    // Only active for chapter > 1
    if let Some(1) = maybe_num {
        return Err(HttpError(AppError::NotFound(
            "Spoiler-free catch-up recap is only available for chapters > 1.".to_string(),
        )));
    }

    let maybe_recap = query_chapter_recap(&state.db, book_id, chapter_id)
        .await
        .map_err(HttpError)?;

    match maybe_recap {
        Some(recap_json) => {
            let dto = ChapterRecapDto {
                book_id,
                chapter_id,
                recap: recap_json,
            };
            Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
        }
        None => Err(HttpError(AppError::NotFound(format!(
            "Recap for chapter {chapter_ref} has not been generated yet."
        )))),
    }
}
