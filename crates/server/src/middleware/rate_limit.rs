//! Redis-backed sliding-window rate limiting middleware for public endpoints.

use crate::{error::HttpError, AppState};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use shared::{ApiResponse, AppError};
use std::net::IpAddr;
use std::sync::Arc;

/// Client IP from headers. Proxy headers are honored only when the
/// deployment sets `TRUST_PROXY_HEADERS=true` (edge overwrites them);
/// otherwise every direct client shares the fallback bucket.
pub fn client_ip_from_headers(headers: &HeaderMap, trusted_proxy: bool) -> String {
    if !trusted_proxy {
        return "127.0.0.1".to_string();
    }
    if let Some(cf_ip) = headers
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
    {
        if cf_ip.parse::<IpAddr>().is_ok() {
            return cf_ip.to_string();
        }
    }
    if let Some(real_ip) = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
    {
        if real_ip.parse::<IpAddr>().is_ok() {
            return real_ip.to_string();
        }
    }
    if let Some(forwarded) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        for part in forwarded.split(',') {
            let candidate = part.trim();
            if candidate.parse::<IpAddr>().is_ok() {
                return candidate.to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Client IP for rate limiting. Proxy headers are honored only when the
/// deployment sets `TRUST_PROXY_HEADERS=true` (edge overwrites them);
/// otherwise every direct client shares the fallback bucket.
pub fn extract_client_ip(req: &Request<Body>, trusted_proxy: bool) -> String {
    client_ip_from_headers(req.headers(), trusted_proxy)
}

/// Caps public auth endpoints at 20 req/min per IP.
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req, state.config.server.trust_proxy_headers);
    let redis_key = format!("rate_limit:auth:{client_ip}");
    let max_requests: i64 = 20;
    let window_seconds: i64 = 60;

    // Fail closed: when Redis is unreachable the limit cannot be enforced,
    // so shed auth traffic instead of letting brute-force through.
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
            let _: Result<(), _> = redis_conn.expire(&redis_key, window_seconds).await;
            ttl = window_seconds;
        }

        let reset_seconds = if ttl > 0 { ttl } else { window_seconds };
        let remaining = (max_requests - count).max(0);

        if count > max_requests {
            let mut resp = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ApiResponse::<()>::error(
                    "RATE_LIMITED",
                    &format!(
                        "Rate limit exceeded. Maximum {max_requests} requests per minute allowed."
                    ),
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
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_ip(header: &str, value: &str) -> Request<Body> {
        Request::builder()
            .uri("/")
            .header(header, value)
            .body(Body::empty())
            .expect("valid test request")
    }

    #[test]
    fn untrusted_mode_ignores_spoofed_headers() {
        let req = request_with_ip("cf-connecting-ip", "203.0.113.7");
        assert_eq!(extract_client_ip(&req, false), "127.0.0.1");
    }

    #[test]
    fn trusted_mode_honors_edge_headers() {
        let req = request_with_ip("cf-connecting-ip", "203.0.113.7");
        assert_eq!(extract_client_ip(&req, true), "203.0.113.7");
        let bare = Request::builder().uri("/").body(Body::empty()).unwrap();
        assert_eq!(extract_client_ip(&bare, true), "127.0.0.1");
    }
}
