//! Catalog, search, chapter, and offline bundle endpoints.

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use shared::{
    ApiResponse, BookCatalogQuery, BookDetailDto, BookSearchQuery, BookSearchResultDto,
    BookSummaryDto, ChapterDetailDto, OfflineBundleDto,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::HttpError, AppState};

/// Catalog listing with cursor pagination and filters.
#[utoipa::path(
    get,
    path = "/api/v1/books",
    tag = "Catalog",
    params(BookCatalogQuery),
    responses(
        (status = 200, description = "List of books matching filters", body = ApiResponse<Vec<BookSummaryDto>>)
    )
)]
pub async fn list_books(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BookCatalogQuery>,
) -> Result<Response, HttpError> {
    let books = infra::list_books(&state.db, &query)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(books)).into_response())
}

/// Typo-tolerant lexical search via PostgreSQL 17 GIN Trigram and FTS indexes
#[utoipa::path(
    get,
    path = "/api/v1/books/search",
    tag = "Catalog",
    params(BookSearchQuery),
    responses(
        (status = 200, description = "Ranked search results", body = ApiResponse<Vec<BookSearchResultDto>>),
        (status = 400, description = "Query length less than 2 characters")
    )
)]
pub async fn search_books(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BookSearchQuery>,
) -> Result<Response, HttpError> {
    let results = infra::search_books(&state.db, &query)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(results)).into_response())
}

/// Retrieve comprehensive book details, synopsis, tags, and chapter summaries
#[utoipa::path(
    get,
    path = "/api/v1/books/{id}",
    tag = "Catalog",
    params(
        ("id" = Uuid, Path, description = "Book UUID")
    ),
    responses(
        (status = 200, description = "Book details", body = ApiResponse<BookDetailDto>),
        (status = 404, description = "Book not found or not published")
    )
)]
pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(book_id): Path<Uuid>,
) -> Result<Response, HttpError> {
    let book = infra::get_book_by_id(&state.db, book_id)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(book)).into_response())
}

/// Retrieve full sanitized chapter HTML content by chapter number
#[utoipa::path(
    get,
    path = "/api/v1/books/{id}/chapters/{chapter_number}",
    tag = "Catalog",
    params(
        ("id" = Uuid, Path, description = "Book UUID"),
        ("chapter_number" = i32, Path, description = "1-based chapter number")
    ),
    responses(
        (status = 200, description = "Chapter content", body = ApiResponse<ChapterDetailDto>),
        (status = 404, description = "Chapter or book not found")
    )
)]
pub async fn get_chapter(
    State(state): State<Arc<AppState>>,
    Path((book_id, chapter_number)): Path<(Uuid, i32)>,
) -> Result<Response, HttpError> {
    let chapter = infra::get_chapter_by_number(&state.db, book_id, chapter_number)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(chapter)).into_response())
}

/// Full book metadata plus chapters for offline caching.
#[utoipa::path(
    get,
    path = "/api/v1/books/{id}/offline-bundle",
    tag = "Catalog",
    params(
        ("id" = Uuid, Path, description = "Book UUID")
    ),
    responses(
        (status = 200, description = "Offline bundle", body = ApiResponse<OfflineBundleDto>),
        (status = 404, description = "Book not found")
    )
)]
pub async fn get_offline_bundle(
    State(state): State<Arc<AppState>>,
    Path(book_id): Path<Uuid>,
) -> Result<Response, HttpError> {
    let bundle = infra::get_offline_bundle(&state.db, book_id)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(bundle)).into_response())
}

/// Assembles public catalog and chapter access routes
pub fn books_routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/", axum::routing::get(list_books))
        .route("/search", axum::routing::get(search_books))
        .route("/{id}", axum::routing::get(get_book))
        .route(
            "/{id}/chapters/{chapter_number}",
            axum::routing::get(get_chapter),
        )
        .route(
            "/{id}/offline-bundle",
            axum::routing::get(get_offline_bundle),
        )
}
