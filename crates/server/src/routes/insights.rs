//! Atomic insight cards and spoiler-free chapter recap handlers.
//!
//! Endpoints:
//! - GET /api/v1/books/{book_id}/chapters/{chapter_ref}/atomic-cards  — Chapter atomic insight cards
//! - GET /api/v1/books/{book_id}/chapters/{chapter_ref}/recap         — Spoiler-free catch-up recap
//!
//! `{chapter_ref}` supports both chapter numbers (e.g. `2`) and chapter UUIDs for maximum client flexibility.
//! Both endpoints read from the `tldr_cache` table with sub-5ms latency on cache hit.

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

// ---------------------------------------------------------------------------
// Helper: Resolve chapter number or UUID to chapter UUID
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Router builder
// ---------------------------------------------------------------------------

/// Sub-router for AI insight endpoints, mounted at /api/v1/books and /api/books.
/// These endpoints are public-read (no auth required for cache-hit reads).
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

// ---------------------------------------------------------------------------
// Chapter Atomic Insight Cards
// ---------------------------------------------------------------------------

/// Retrieve precomputed atomic insight cards for a chapter from `tldr_cache`.
///
/// Accepts either chapter UUID or chapter number in the path (e.g. `/chapters/1/atomic-cards`).
/// Returns structured JSON with categories: `key_concept`, `notable_quote`,
/// `historical_context`. Zero token cost on cache hit (sub-5ms).
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

// ---------------------------------------------------------------------------
// Spoiler-Free Catch-up Recap
// ---------------------------------------------------------------------------

/// Retrieve a spoiler-free catch-up summary for a chapter from `tldr_cache`.
///
/// Accepts either chapter number (e.g. `/chapters/2/recap`) or chapter UUID.
/// Summarizes key events from prior chapters. Only active for chapters > 1;
/// for chapter 1 the recap is unavailable and a 404 is returned.
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
