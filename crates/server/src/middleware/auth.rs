//! JWT Authentication extractor for Axum handlers with revocation enforcement.

use crate::{error::HttpError, AppState};
use axum::{extract::FromRequestParts, http::request::Parts};
use infra::{
    is_session_revoked, is_token_blacklisted, is_user_token_revoked,
    security::jwt::verify_access_token, REDIS_RETRY_AFTER_SECS,
};
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

fn auth_state_unavailable(context: &str, error: AppError) -> HttpError {
    tracing::warn!(target: "server::auth", error = %error, "{context}; rejecting request");
    HttpError(AppError::ServiceUnavailable {
        retry_after: REDIS_RETRY_AFTER_SECS,
    })
}

/// Authenticated user extracted from a valid JWT Bearer token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub jti: Uuid,
    pub sid: Option<Uuid>,
    pub exp: usize,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = HttpError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                HttpError(AppError::Unauthorized(
                    "Missing Authorization header".to_string(),
                ))
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            HttpError(AppError::Unauthorized(
                "Invalid Authorization scheme. Bearer expected".to_string(),
            ))
        })?;

        let claims =
            verify_access_token(token, state.config.jwt_secret()).map_err(HttpError::from)?;

        // Fail closed when auth state cannot be verified: a revoked token must
        // never be accepted just because Redis is unreachable. The answer is
        // 503, not 401, so clients keep a token that is still valid.
        let mut redis_conn = state
            .get_redis_conn()
            .await
            .map_err(|e| auth_state_unavailable("Redis unavailable", e))?;

        if is_token_blacklisted(&mut redis_conn, claims.jti)
            .await
            .map_err(|e| auth_state_unavailable("Blacklist check failed", e))?
        {
            return Err(HttpError(AppError::Unauthorized(
                "Access token has been revoked. Please log in again.".to_string(),
            )));
        }

        if let Some(sid) = claims.sid {
            if is_session_revoked(&mut redis_conn, sid)
                .await
                .map_err(|e| auth_state_unavailable("Session check failed", e))?
            {
                return Err(HttpError(AppError::Unauthorized(
                    "Access token has been revoked. Please log in again.".to_string(),
                )));
            }
        }

        if is_user_token_revoked(&mut redis_conn, claims.sub, claims.iat)
            .await
            .map_err(|e| auth_state_unavailable("Revocation check failed", e))?
        {
            return Err(HttpError(AppError::Unauthorized(
                "Session has been invalidated. Please log in again.".to_string(),
            )));
        }

        Ok(AuthUser {
            id: claims.sub,
            email: claims.email,
            role: claims.role,
            jti: claims.jti,
            sid: claims.sid,
            exp: claims.exp,
        })
    }
}
