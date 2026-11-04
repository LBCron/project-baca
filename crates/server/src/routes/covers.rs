//! Cover images, served from the covers bucket under the key the worker
//! stores on the book (`covers/<book uuid>.<ext>`).

use crate::{error::HttpError, AppState};
use axum::{
    extract::{Path, State},
    http::{header, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use shared::AppError;
use std::sync::Arc;

/// Covers are addressed by book id and rewritten only by a fresh ingestion,
/// so browsers may keep them for a year.
const COVER_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

pub fn covers_routes() -> Router<Arc<AppState>> {
    Router::new().route("/{file}", get(get_cover))
}

/// `<36 lowercase hex-or-dash chars>.(webp|jpg|jpeg|png)`: the only shape
/// the worker writes, so no other key in the bucket is reachable from the web.
fn is_cover_file_name(file: &str) -> bool {
    let Some((stem, ext)) = file.rsplit_once('.') else {
        return false;
    };
    stem.len() == 36
        && stem
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'-'))
        && matches!(ext, "webp" | "jpg" | "jpeg" | "png")
}

/// Media type from the file's allowlisted extension, never from what the
/// bucket recorded: an object stored as `image/svg+xml` is still served as a
/// raster type the browser will not run.
fn cover_content_type(file: &str) -> &'static str {
    match file.rsplit_once('.').map(|(_, ext)| ext) {
        Some("webp") => "image/webp",
        Some("png") => "image/png",
        _ => "image/jpeg",
    }
}

/// Cover image bytes for a published book. Public, cached immutably, 404 for
/// anything that is not the stored cover of a published book.
#[utoipa::path(
    get,
    path = "/api/v1/covers/{file}",
    tag = "Catalog",
    params(
        ("file" = String, Path, description = "Cover file name as stored on the book: <book uuid>.webp|jpg|jpeg|png")
    ),
    responses(
        (status = 200, description = "Cover image", content_type = "image/webp"),
        (status = 404, description = "No such cover, or its book is not published")
    )
)]
pub async fn get_cover(
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
) -> Result<Response, HttpError> {
    let not_found = || HttpError(AppError::NotFound("Cover not found".to_string()));
    let Some(book_id) = is_cover_file_name(&file)
        .then(|| file.rsplit_once('.'))
        .flatten()
        .and_then(|(stem, _)| stem.parse::<uuid::Uuid>().ok())
    else {
        return Err(not_found());
    };
    // A draft or archived book's cover is as invisible as the book: a
    // takedown takes the cover down with it.
    infra::ensure_book_published(&state.db, book_id)
        .await
        .map_err(|e| match e {
            AppError::NotFound(_) => not_found(),
            other => HttpError(other),
        })?;
    let storage = state.storage.as_ref().ok_or_else(|| {
        HttpError(AppError::Internal(
            "Object storage unavailable; covers cannot be served".to_string(),
        ))
    })?;
    let bytes = storage
        .get_cover(&format!("covers/{file}"))
        .await
        .map_err(|e| match e {
            AppError::NotFound(_) => not_found(),
            other => HttpError(other),
        })?;

    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static(cover_content_type(&file)),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static(COVER_CACHE_CONTROL),
            ),
            // The reader may load covers from another origin in development.
            (
                HeaderName::from_static("cross-origin-resource-policy"),
                HeaderValue::from_static("cross-origin"),
            ),
        ],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_uuid_named_images() {
        assert!(is_cover_file_name(
            "a0000000-0000-4000-8000-000000000001.webp"
        ));
        assert!(is_cover_file_name(
            "a0000000-0000-4000-8000-000000000001.jpeg"
        ));
        assert!(!is_cover_file_name(
            "A0000000-0000-4000-8000-000000000001.webp"
        ));
        assert!(!is_cover_file_name("../raw-epubs/x.epub"));
        assert!(!is_cover_file_name(
            "a0000000-0000-4000-8000-000000000001.svg"
        ));
        assert!(!is_cover_file_name(
            "a0000000-0000-4000-8000-00000000001.webp"
        ));
        assert!(!is_cover_file_name("webp"));
    }

    #[test]
    fn content_type_comes_from_the_extension_alone() {
        assert_eq!(cover_content_type("x.webp"), "image/webp");
        assert_eq!(cover_content_type("x.png"), "image/png");
        assert_eq!(cover_content_type("x.jpg"), "image/jpeg");
        assert_eq!(cover_content_type("x.jpeg"), "image/jpeg");
    }
}
