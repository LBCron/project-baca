//! HTTP Error mapping for Project Baca Axum REST API.

use axum::{
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use shared::{ApiResponse, AppError};

/// Postgres's refusal of a NUL byte in UTF-8 text (SQLSTATE 22021).
const NUL_IN_TEXT: &str = "invalid byte sequence for encoding \"UTF8\": 0x00";

/// Wrapper converting internal `AppError` into standardized Axum HTTP responses
#[derive(Debug)]
pub struct HttpError(pub AppError);

impl From<AppError> for HttpError {
    fn from(err: AppError) -> Self {
        HttpError(err)
    }
}

impl From<sea_orm::DbErr> for HttpError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!(target: "server::database", error = %err, "Database error encountered");
        HttpError(AppError::Database(format!("Database error: {err}")))
    }
}

impl From<validator::ValidationErrors> for HttpError {
    fn from(err: validator::ValidationErrors) -> Self {
        let mut field_map = serde_json::Map::new();
        let mut first_msg = "Validation failed".to_string();
        let mut found_first = false;

        for (field, errors) in err.field_errors() {
            let messages: Vec<String> = errors
                .iter()
                .map(|e| {
                    let msg = e
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Invalid value for {field}"));
                    if !found_first {
                        first_msg = msg.clone();
                        found_first = true;
                    }
                    msg
                })
                .collect();
            field_map.insert(field.to_string(), serde_json::Value::from(messages));
        }

        HttpError(AppError::ValidationDetailed {
            message: first_msg,
            details: serde_json::Value::Object(field_map),
        })
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match &self.0 {
            AppError::ValidationDetailed { message, details } => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    shared::error_code::VALIDATION_FAILED,
                    message,
                    Some(details.clone()),
                )),
            )
                .into_response(),
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    shared::error_code::VALIDATION_FAILED,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    shared::error_code::BAD_REQUEST,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error(
                    shared::error_code::UNAUTHORIZED,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error(
                    shared::error_code::FORBIDDEN,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(
                    shared::error_code::NOT_FOUND,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                Json(ApiResponse::<()>::error(
                    shared::error_code::CONFLICT,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::PayloadTooLarge(msg) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(ApiResponse::<()>::error(
                    shared::error_code::PAYLOAD_TOO_LARGE,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::QuotaExceeded(msg) => (
                StatusCode::CONFLICT,
                Json(ApiResponse::<()>::error(
                    shared::error_code::QUOTA_EXCEEDED,
                    msg,
                    None,
                )),
            )
                .into_response(),
            AppError::RateLimited { retry_after } => {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiResponse::<()>::error(
                        shared::error_code::RATE_LIMITED,
                        &format!("Rate limit exceeded. Try again in {retry_after} seconds"),
                        None,
                    )),
                )
                    .into_response();
                if let Ok(v) = HeaderValue::from_str(&retry_after.to_string()) {
                    resp.headers_mut().insert(header::RETRY_AFTER, v);
                }
                resp
            }
            // Postgres text cannot hold NUL; a reader typed it, not a fault here.
            AppError::Database(msg) if msg.contains(NUL_IN_TEXT) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    shared::error_code::BAD_REQUEST,
                    "Text must not contain a NUL character",
                    None,
                )),
            )
                .into_response(),
            AppError::Database(msg) => {
                tracing::error!(target: "server::database", error = %msg, "Database error encountered");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        shared::error_code::DATABASE_ERROR,
                        "A database error occurred. Please try again later.",
                        None,
                    )),
                )
                    .into_response()
            }
            AppError::Internal(msg) => {
                tracing::error!(target: "server::internal", error = %msg, "Internal application error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        shared::error_code::INTERNAL_ERROR,
                        "An internal server error occurred. Please try again later.",
                        None,
                    )),
                )
                    .into_response()
            }
            AppError::ExternalService(msg) => {
                tracing::error!(target: "server::external", error = %msg, "External service communication error");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(ApiResponse::<()>::error(
                        shared::error_code::EXTERNAL_SERVICE_ERROR,
                        "External service temporarily unavailable. Please try again later.",
                        None,
                    )),
                )
                    .into_response()
            }
            AppError::ServiceUnavailable { retry_after } => {
                let mut resp = (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::<()>::error(
                        shared::error_code::SERVICE_UNAVAILABLE,
                        &format!(
                            "Service temporarily unavailable. Try again in {retry_after} seconds"
                        ),
                        None,
                    )),
                )
                    .into_response();
                if let Ok(v) = HeaderValue::from_str(&retry_after.to_string()) {
                    resp.headers_mut().insert(header::RETRY_AFTER, v);
                }
                resp
            }
        }
    }
}

/// Longest rejection text read back; axum's own messages are one line.
const REJECTION_TEXT_LIMIT: usize = 4096;

/// Axum answers extractor rejections (path, query, content type, JSON,
/// multipart) and a wrong method in plain text before any handler runs.
/// Every client parses one envelope, so those become it here, with their
/// status and headers (`Allow`) kept. Handler errors are JSON already and
/// pass through.
pub async fn envelope_rejections(req: Request, next: Next) -> Response {
    let resp = next.run(req).await;
    let is_json = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if !resp.status().is_client_error() || is_json {
        return resp;
    }
    let (mut parts, body) = resp.into_parts();
    let text = axum::body::to_bytes(body, REJECTION_TEXT_LIMIT)
        .await
        .map(|b| String::from_utf8_lossy(&b).trim().to_string())
        .unwrap_or_default();
    let message = if text.is_empty() {
        parts
            .status
            .canonical_reason()
            .unwrap_or("Request rejected")
            .to_string()
    } else {
        text
    };
    let code = match parts.status {
        StatusCode::METHOD_NOT_ALLOWED => shared::error_code::METHOD_NOT_ALLOWED,
        StatusCode::PAYLOAD_TOO_LARGE => shared::error_code::PAYLOAD_TOO_LARGE,
        StatusCode::UNPROCESSABLE_ENTITY => shared::error_code::VALIDATION_FAILED,
        _ => shared::error_code::BAD_REQUEST,
    };
    parts.headers.remove(header::CONTENT_LENGTH);
    parts.headers.remove(header::CONTENT_TYPE);
    let envelope = Json(ApiResponse::<()>::error(code, &message, None)).into_response();
    let (envelope_parts, envelope_body) = envelope.into_parts();
    parts.headers.extend(envelope_parts.headers);
    Response::from_parts(parts, envelope_body)
}
