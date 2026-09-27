//! HTTP Error mapping for Project Baca Axum REST API.

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use shared::{ApiResponse, AppError};

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
                    "VALIDATION_FAILED",
                    message,
                    Some(details.clone()),
                )),
            )
                .into_response(),
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("VALIDATION_FAILED", msg, None)),
            )
                .into_response(),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("BAD_REQUEST", msg, None)),
            )
                .into_response(),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", msg, None)),
            )
                .into_response(),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error("FORBIDDEN", msg, None)),
            )
                .into_response(),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("NOT_FOUND", msg, None)),
            )
                .into_response(),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                Json(ApiResponse::<()>::error("CONFLICT", msg, None)),
            )
                .into_response(),
            AppError::RateLimited { retry_after } => {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiResponse::<()>::error(
                        "RATE_LIMITED",
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
            AppError::Database(msg) => {
                tracing::error!(target: "server::database", error = %msg, "Database error encountered");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        "DATABASE_ERROR",
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
                        "INTERNAL_ERROR",
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
                        "EXTERNAL_SERVICE_ERROR",
                        "External service temporarily unavailable. Please try again later.",
                        None,
                    )),
                )
                    .into_response()
            }
        }
    }
}
