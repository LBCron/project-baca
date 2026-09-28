//! Sliding-window AI rate limit (10 req/min): users by UUID, guests by IP.

use crate::{error::HttpError, AppState};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use infra::verify_access_token;
use redis::AsyncCommands;
use shared::{ApiResponse, AppError};
use std::net::IpAddr;
use std::sync::Arc;

const AI_MAX_REQUESTS: i64 = 10;
const AI_WINDOW_SECONDS: i64 = 60;

fn get_client_ip(req: &Request<Body>, trusted_proxy: bool) -> String {
    if !trusted_proxy {
        return "127.0.0.1".to_string();
    }
    // Cloudflare connecting IP
    if let Some(cf_ip) = req
        .headers()
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
    {
        if cf_ip.parse::<IpAddr>().is_ok() {
            return cf_ip.to_string();
        }
    }

    // Standard X-Real-IP reverse proxy header
    if let Some(real_ip) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
    {
        if real_ip.parse::<IpAddr>().is_ok() {
            return real_ip.to_string();
        }
    }

    // X-Forwarded-For header
    if let Some(forwarded) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        for part in forwarded.split(',') {
            let candidate = part.trim();
            if candidate.parse::<IpAddr>().is_ok() {
                return candidate.to_string();
            }
        }
    }

    "127.0.0.1".to_string()
}

/// Sliding-window limiter: users by ID, guests by IP.
pub async fn ai_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let maybe_user_id: Option<String> = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            verify_access_token(token, state.config.jwt_secret())
                .ok()
                .map(|c| c.sub.to_string())
        });

    let redis_key = match &maybe_user_id {
        Some(user_id) => format!("rate_limit:ai:user:{user_id}"),
        None => {
            let ip = get_client_ip(&req, state.config.server.trust_proxy_headers);
            format!("rate_limit:ai:guest:{ip}")
        }
    };

    // Fail closed: each search burns embedding quota, so an unenforceable
    // limit must shed AI traffic instead of opening unlimited spend.
    let mut redis_conn = match state.get_redis_conn().await {
        Ok(conn) => conn,
        Err(_) => {
            return HttpError(AppError::ServiceUnavailable { retry_after: 60 }).into_response()
        }
    };
    let count: i64 = match redis_conn.incr(&redis_key, 1).await {
        Ok(c) => c,
        Err(_) => {
            return HttpError(AppError::ServiceUnavailable { retry_after: 60 }).into_response()
        }
    };
    {
        let mut ttl: i64 = redis_conn.ttl(&redis_key).await.unwrap_or(-1);
        if ttl <= 0 {
            let _: Result<(), _> = redis_conn.expire(&redis_key, AI_WINDOW_SECONDS).await;
            ttl = AI_WINDOW_SECONDS;
        }

        let reset_seconds = if ttl > 0 { ttl } else { AI_WINDOW_SECONDS };
        let remaining = (AI_MAX_REQUESTS - count).max(0);

        if count > AI_MAX_REQUESTS {
            let mut resp = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ApiResponse::<()>::error(
                    "AI_RATE_LIMITED",
                    &format!(
                        "AI rate limit exceeded. Maximum {AI_MAX_REQUESTS} semantic \
                             search requests per minute allowed."
                    ),
                    None,
                )),
            )
                .into_response();

            let headers = resp.headers_mut();
            if let Ok(v) = HeaderValue::from_str(&AI_MAX_REQUESTS.to_string()) {
                headers.insert(HeaderName::from_static("x-ratelimit-limit"), v);
            }
            headers.insert(
                HeaderName::from_static("x-ratelimit-remaining"),
                HeaderValue::from_static("0"),
            );
            if let Ok(v) = HeaderValue::from_str(&reset_seconds.to_string()) {
                headers.insert(HeaderName::from_static("x-ratelimit-reset"), v.clone());
                headers.insert(header::RETRY_AFTER, v);
            }
            return resp;
        }

        let mut response = next.run(req).await;
        let headers = response.headers_mut();
        if let Ok(v) = HeaderValue::from_str(&AI_MAX_REQUESTS.to_string()) {
            headers.insert(HeaderName::from_static("x-ratelimit-limit"), v);
        }
        if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
            headers.insert(HeaderName::from_static("x-ratelimit-remaining"), v);
        }
        if let Ok(v) = HeaderValue::from_str(&reset_seconds.to_string()) {
            headers.insert(HeaderName::from_static("x-ratelimit-reset"), v);
        }
        response
    }
}
