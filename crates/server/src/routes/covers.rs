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

/// Media type from the stored object, falling back to the extension when the
/// bucket did not record an image type.
fn cover_content_type(file: &str, stored: Option<String>) -> String {
    stored
        .filter(|ct| ct.starts_with("image/"))
        .unwrap_or_else(|| {
            match file.rsplit_once('.').map(|(_, ext)| ext) {
                Some("webp") => "image/webp",
                Some("png") => "image/png",
                _ => "image/jpeg",
            }
            .to_string()
        })
}

/// Cover image bytes for a book. Public, cached immutably, 404 for anything
/// that is not a stored cover.
#[utoipa::path(
    get,
    path = "/api/v1/covers/{file}",
    tag = "Catalog",
    params(
        ("file" = String, Path, description = "Cover file name as stored on the book: <book uuid>.webp|jpg|jpeg|png")
    ),
    responses(
        (status = 200, description = "Cover image", content_type = "image/webp"),
        (status = 404, description = "No such cover")
    )
)]
pub async fn get_cover(
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
) -> Result<Response, HttpError> {
    let not_found = || HttpError(AppError::NotFound("Cover not found".to_string()));
    if !is_cover_file_name(&file) {
        return Err(not_found());
    }
    let storage = state.storage.as_ref().ok_or_else(|| {
        HttpError(AppError::Internal(
            "Object storage unavailable; covers cannot be served".to_string(),
        ))
    })?;
    let object = storage
        .get_cover(&format!("covers/{file}"))
        .await
        .map_err(|e| match e {
            AppError::NotFound(_) => not_found(),
            other => HttpError(other),
        })?;
    let content_type = HeaderValue::from_str(&cover_content_type(&file, object.content_type))
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
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
        object.bytes,
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
    fn content_type_prefers_stored_image_type() {
        assert_eq!(
            cover_content_type("x.webp", Some("image/png".to_string())),
            "image/png"
        );
        assert_eq!(
            cover_content_type("x.webp", Some("text/html".to_string())),
            "image/webp"
        );
        assert_eq!(cover_content_type("x.jpg", None), "image/jpeg");
    }
}
