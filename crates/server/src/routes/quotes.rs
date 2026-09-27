//! Quote search, saved quotes, and vintage card endpoints.

use crate::{error::HttpError, middleware::AuthUser, AppState};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use infra::{
    get_book_by_id, get_chapter_book_id, get_saved_quote_by_id,
    list_saved_quotes as repo_list_saved_quotes, save_quote as repo_save_quote,
    search_quotes_by_embedding, verify_access_token,
};
use shared::{
    ApiResponse, AppError, QuoteSearchRequest, QuoteSearchResultDto, SaveQuoteRequest,
    SavedQuoteResponseDto,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub fn quotes_routes() -> Router<Arc<AppState>> {
    Router::new().route("/{book_id}/quotes/search", post(search_book_quotes))
}

pub fn saved_quotes_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/save", post(handle_save_quote))
        .route("/{quote_id}/card", get(get_quote_card))
        .route("/", get(handle_list_saved_quotes))
}

/// Relevant quotes within one book, ranked by cosine similarity. Open to guests.
#[utoipa::path(
    post,
    path = "/api/v1/books/{book_id}/quotes/search",
    params(
        ("book_id" = Uuid, Path, description = "UUID of the book to search within")
    ),
    request_body = QuoteSearchRequest,
    responses(
        (status = 200, description = "Semantically relevant quotes ordered by cosine similarity", body = Vec<QuoteSearchResultDto>),
        (status = 400, description = "Validation error on query payload"),
        (status = 404, description = "Book not found"),
        (status = 429, description = "AI rate limit exceeded (10 req/min per user or IP)"),
        (status = 500, description = "Embedding provider or database error")
    ),
    tag = "Semantic Search"
)]
pub async fn search_book_quotes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(book_id): Path<Uuid>,
    Json(payload): Json<QuoteSearchRequest>,
) -> Result<impl IntoResponse, HttpError> {
    payload
        .validate()
        .map_err(|e| HttpError(AppError::ValidationError(e.to_string())))?;

    // Fail fast on an unknown book before spending embedding budget on a query
    // that can only return [].
    get_book_by_id(&state.db, book_id)
        .await
        .map_err(HttpError)?;

    // Inspect the token for future search telemetry.
    let _maybe_user_id: Option<Uuid> = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            verify_access_token(token, state.config.jwt_secret())
                .ok()
                .map(|c| c.sub)
        });

    let limit = payload.limit.unwrap_or(5);

    let embedding = state
        .embedding
        .embed_text(&payload.query)
        .await
        .map_err(HttpError)?;

    let rows = search_quotes_by_embedding(&state.db, book_id, &embedding, limit)
        .await
        .map_err(HttpError)?;

    let results: Vec<QuoteSearchResultDto> = rows
        .into_iter()
        .map(|r| QuoteSearchResultDto {
            chunk_id: r.chunk_id,
            chapter_number: r.chapter_number,
            chapter_title: r.chapter_title,
            cfi_range: r.cfi_range,
            content: r.content,
            similarity_score: r.similarity_score,
        })
        .collect();

    Ok((StatusCode::OK, Json(ApiResponse::success(results))))
}

// Saved Quotes Management & Vintage Quote Card

/// Saves a quote; guests get guest_mode, members persist with a card URL.
#[utoipa::path(
    post,
    path = "/api/v1/quotes/save",
    request_body = SaveQuoteRequest,
    responses(
        (status = 201, description = "Quote saved successfully"),
        (status = 400, description = "Validation error or chapter does not belong to the book"),
        (status = 404, description = "Unknown book or chapter"),
        (status = 500, description = "Database error")
    ),
    security(("BearerAuth" = [])),
    tag = "Semantic Search"
)]
pub async fn handle_save_quote(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SaveQuoteRequest>,
) -> Result<impl IntoResponse, HttpError> {
    payload
        .validate()
        .map_err(|e| HttpError(AppError::ValidationError(e.to_string())))?;

    let maybe_user_id: Option<Uuid> = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            verify_access_token(token, state.config.jwt_secret())
                .ok()
                .map(|c| c.sub)
        });

    let Some(user_id) = maybe_user_id else {
        return Ok((
            StatusCode::CREATED,
            Json(ApiResponse::success(serde_json::json!({
                "saved": false,
                "reason": "guest_mode"
            }))),
        ));
    };

    let id = Uuid::new_v4();
    let card_url = format!("/api/v1/quotes/{id}/card");

    // Validate the (book_id, chapter_id) pair: 404 for unknown ids, 400 for
    // cross-book mismatches — never a raw 500 FK violation.
    get_book_by_id(&state.db, payload.book_id)
        .await
        .map_err(HttpError)?;
    let chapter_book_id = get_chapter_book_id(&state.db, payload.chapter_id)
        .await
        .map_err(HttpError)?;
    if chapter_book_id != payload.book_id {
        return Err(HttpError(AppError::ValidationError(format!(
            "Chapter {} does not belong to book {}",
            payload.chapter_id, payload.book_id
        ))));
    }

    repo_save_quote(
        &state.db,
        id,
        user_id,
        payload.book_id,
        payload.chapter_id,
        &payload.quote_text,
        Some(&card_url),
    )
    .await
    .map_err(HttpError)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(serde_json::json!({
            "id": id,
            "saved": true,
            "image_card_url": card_url
        }))),
    ))
}

/// Saved quotes of the authenticated user, most recent first.
#[utoipa::path(
    get,
    path = "/api/v1/quotes",
    responses(
        (status = 200, description = "List of saved quotes ordered by recency", body = Vec<SavedQuoteResponseDto>),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Database error")
    ),
    security(("BearerAuth" = [])),
    tag = "Semantic Search"
)]
pub async fn handle_list_saved_quotes(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, HttpError> {
    let quotes = repo_list_saved_quotes(&state.db, auth_user.id)
        .await
        .map_err(HttpError)?;

    let dtos: Vec<SavedQuoteResponseDto> = quotes
        .into_iter()
        .map(|q| SavedQuoteResponseDto {
            id: q.id,
            book_id: q.book_id,
            chapter_id: q.chapter_id,
            quote_text: q.quote_text,
            image_card_url: q.image_card_url,
            created_at: q.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(ApiResponse::success(dtos))))
}

// Vintage Quote Card Export (SVG)

fn render_vintage_quote_svg(quote_text: &str, book_title: &str, author: &str) -> String {
    let escaped_quote = quote_text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let escaped_title = book_title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let escaped_author = author
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 630" width="1200" height="630">
  <defs>
    <style>
      .bg {{ fill: #F9F6F0; }}
      .border-outer {{ fill: none; stroke: #2B2625; stroke-width: 3; }}
      .border-inner {{ fill: none; stroke: #9D5A3C; stroke-width: 1; stroke-dasharray: 6 3; }}
      .brand {{ font-family: 'Courier New', Courier, monospace; font-size: 13px; fill: #9D5A3C; text-anchor: middle; letter-spacing: 5px; }}
      .ornament {{ font-family: Georgia, serif; font-size: 24px; fill: #9D5A3C; text-anchor: middle; }}
      .quote-mark {{ font-family: Georgia, serif; font-size: 80px; fill: #9D5A3C; opacity: 0.25; }}
      .quote-text {{ font-family: Georgia, serif; font-size: 30px; font-style: italic; fill: #2B2625; text-anchor: middle; }}
      .meta {{ font-family: 'Courier New', Courier, monospace; font-size: 18px; fill: #5A524F; text-anchor: middle; letter-spacing: 2px; text-transform: uppercase; }}
    </style>
  </defs>
  <rect width="1200" height="630" class="bg" />
  <rect x="30" y="30" width="1140" height="570" class="border-outer" />
  <rect x="42" y="42" width="1116" height="546" class="border-inner" />
  <text x="600" y="90" class="brand">PROJECT BACA &#8226; CLASSIC LITERARY COLLECTION</text>
  <text x="600" y="130" class="ornament">&#10022;</text>
  <text x="120" y="240" class="quote-mark">&#8220;</text>
  <text x="600" y="310" class="quote-text">{escaped_quote}</text>
  <text x="1080" y="420" class="quote-mark" text-anchor="end">&#8221;</text>
  <text x="600" y="490" class="ornament">&#10022;</text>
  <text x="600" y="535" class="meta">&#8212; {escaped_author}, &#8220;{escaped_title}&#8221; &#8212;</text>
</svg>"#
    )
}

/// Vintage SVG quote card for social sharing.
#[utoipa::path(
    get,
    path = "/api/v1/quotes/{quote_id}/card",
    params(
        ("quote_id" = Uuid, Path, description = "UUID of the saved quote to render")
    ),
    responses(
        (status = 200, description = "Vintage quote card SVG image", content_type = "image/svg+xml"),
        (status = 404, description = "Quote not found"),
        (status = 500, description = "Database error")
    ),
    tag = "Semantic Search"
)]
pub async fn get_quote_card(
    State(state): State<Arc<AppState>>,
    Path(quote_id): Path<Uuid>,
) -> Result<Response, HttpError> {
    let quote = get_saved_quote_by_id(&state.db, quote_id)
        .await
        .map_err(HttpError)?
        .ok_or_else(|| HttpError(AppError::NotFound("Saved quote not found".to_string())))?;

    let book = get_book_by_id(&state.db, quote.book_id).await.ok();

    let title = book
        .as_ref()
        .map(|b| b.title.as_str())
        .unwrap_or("Classic Literature");
    let author = book
        .as_ref()
        .map(|b| b.author.as_str())
        .unwrap_or("Anonymous");

    let svg = render_vintage_quote_svg(&quote.quote_text, title, author);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        svg,
    )
        .into_response())
}
