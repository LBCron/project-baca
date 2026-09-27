//! Redis-backed sliding-window rate limiting middleware for public endpoints.

use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use shared::ApiResponse;
use std::net::IpAddr;
use std::sync::Arc;

/// Extracts and validates client IP address from trusted edge headers or peer connection
pub fn extract_client_ip(req: &Request<Body>) -> String {
    // 1. Cloudflare CF-Connecting-IP header (highest priority behind edge WAF)
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

    // 2. Standard X-Real-IP reverse proxy header
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

    // 3. X-Forwarded-For header (validate each entry, take first valid IP address)
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

/// Limits requests on public authentication endpoints (maximum 20 requests per minute per IP)
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);
    let redis_key = format!("rate_limit:auth:{client_ip}");
    let max_requests: i64 = 20;
    let window_seconds: i64 = 60;

    if let Ok(mut redis_conn) = state.get_redis_conn().await {
        let count: Result<i64, _> = redis_conn.incr(&redis_key, 1).await;
        if let Ok(c) = count {
            let mut ttl: i64 = redis_conn.ttl(&redis_key).await.unwrap_or(-1);
            if ttl <= 0 {
                let _: Result<(), _> = redis_conn.expire(&redis_key, window_seconds).await;
                ttl = window_seconds;
            }

            let reset_seconds = if ttl > 0 { ttl } else { window_seconds };
            let remaining = (max_requests - c).max(0);

            if c > max_requests {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiResponse::<()>::error(
                        "RATE_LIMITED",
                        &format!("Rate limit exceeded. Maximum {max_requests} requests per minute allowed."),
                        None,
                    )),
                )
                    .into_response();

                let headers = resp.headers_mut();
                if let Ok(v) = HeaderValue::from_str(&max_requests.to_string()) {
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
            if let Ok(v) = HeaderValue::from_str(&max_requests.to_string()) {
                headers.insert(HeaderName::from_static("x-ratelimit-limit"), v);
            }
            if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
                headers.insert(HeaderName::from_static("x-ratelimit-remaining"), v);
            }
            if let Ok(v) = HeaderValue::from_str(&reset_seconds.to_string()) {
                headers.insert(HeaderName::from_static("x-ratelimit-reset"), v);
            }
            return response;
        }
    }

    next.run(req).await
}
