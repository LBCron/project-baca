//! Redis-backed fixed-window rate limiting shared by the auth, AI, and public
//! catalog policies. One enforcement engine, three thin middlewares.

use crate::{error::HttpError, AppState};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use infra::verify_access_token;
use redis::AsyncCommands;
use shared::{ApiResponse, AppError};
use std::net::IpAddr;
use std::sync::Arc;

/// Window policy: how many requests a subject may make per window and how
/// the refusal is reported.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitPolicy {
    pub key_prefix: &'static str,
    pub max_requests: i64,
    pub window_seconds: i64,
    pub error_code: &'static str,
    pub subject_label: &'static str,
}

/// Public auth endpoints: 20 req/min per IP.
pub const AUTH_POLICY: RateLimitPolicy = RateLimitPolicy {
    key_prefix: "rate_limit:auth",
    max_requests: 20,
    window_seconds: 60,
    error_code: "RATE_LIMITED",
    subject_label: "requests",
};

/// Embedding-backed search: 10 req/min per user, or per IP for guests.
pub const AI_POLICY: RateLimitPolicy = RateLimitPolicy {
    key_prefix: "rate_limit:ai",
    max_requests: 10,
    window_seconds: 60,
    error_code: "AI_RATE_LIMITED",
    subject_label: "semantic search requests",
};

/// Unauthenticated catalog and insight reads; the cap comes from config.
pub fn public_policy(max_requests: u32) -> RateLimitPolicy {
    RateLimitPolicy {
        key_prefix: "rate_limit:public",
        max_requests: i64::from(max_requests),
        window_seconds: 60,
        error_code: "RATE_LIMITED",
        subject_label: "requests",
    }
}

/// Client IP from headers. Proxy headers are honored only when the
/// deployment sets `TRUST_PROXY_HEADERS=true` (edge overwrites them);
/// otherwise every direct client shares the fallback bucket.
pub fn client_ip_from_headers(headers: &HeaderMap, trusted_proxy: bool) -> String {
    if !trusted_proxy {
        return "127.0.0.1".to_string();
    }
    let single_valued = ["cf-connecting-ip", "x-real-ip"];
    for name in single_valued {
        if let Some(candidate) = headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| s.parse::<IpAddr>().is_ok())
        {
            return candidate.to_string();
        }
    }
    if let Some(forwarded) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(candidate) = forwarded
            .split(',')
            .map(str::trim)
            .find(|s| s.parse::<IpAddr>().is_ok())
        {
            return candidate.to_string();
        }
    }
    "127.0.0.1".to_string()
}

/// Client IP for rate limiting (see [`client_ip_from_headers`]).
pub fn extract_client_ip(req: &Request<Body>, trusted_proxy: bool) -> String {
    client_ip_from_headers(req.headers(), trusted_proxy)
}

fn set_limit_headers(headers: &mut HeaderMap, limit: i64, remaining: i64, reset: i64) {
    let pairs = [
        ("x-ratelimit-limit", limit),
        ("x-ratelimit-remaining", remaining),
        ("x-ratelimit-reset", reset),
    ];
    for (name, value) in pairs {
        if let Ok(v) = HeaderValue::from_str(&value.to_string()) {
            headers.insert(HeaderName::from_static(name), v);
        }
    }
}

/// Counts one hit for `subject` and either forwards the request (with RFC
/// rate-limit headers) or answers 429. Fails closed when Redis is
/// unreachable: an unenforceable limit must shed traffic, not open it.
async fn enforce(
    state: &AppState,
    policy: RateLimitPolicy,
    subject: &str,
    req: Request<Body>,
    next: Next,
) -> Response {
    let redis_key = format!("{}:{subject}", policy.key_prefix);

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

    let mut ttl: i64 = redis_conn.ttl(&redis_key).await.unwrap_or(-1);
    if ttl <= 0 {
        let _: Result<(), _> = redis_conn.expire(&redis_key, policy.window_seconds).await;
        ttl = policy.window_seconds;
    }
    let remaining = (policy.max_requests - count).max(0);

    if count > policy.max_requests {
        let mut resp = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiResponse::<()>::error(
                policy.error_code,
                &format!(
                    "Rate limit exceeded. Maximum {} {} per minute allowed.",
                    policy.max_requests, policy.subject_label
                ),
                None,
            )),
        )
            .into_response();
        set_limit_headers(resp.headers_mut(), policy.max_requests, 0, ttl);
        if let Ok(v) = HeaderValue::from_str(&ttl.to_string()) {
            resp.headers_mut().insert(header::RETRY_AFTER, v);
        }
        return resp;
    }

    let mut response = next.run(req).await;
    set_limit_headers(response.headers_mut(), policy.max_requests, remaining, ttl);
    response
}

/// Caps public auth endpoints per IP.
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req, state.config.server.trust_proxy_headers);
    enforce(&state, AUTH_POLICY, &ip, req, next).await
}

/// Caps embedding-backed search: signed-in users by id, guests by IP. The
/// token is only read for bucketing, so revocation is left to the handler.
pub async fn ai_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let user_id = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| verify_access_token(token, state.config.jwt_secret()).ok())
        .map(|claims| claims.sub.to_string());
    let subject = match user_id {
        Some(id) => format!("user:{id}"),
        None => format!(
            "guest:{}",
            extract_client_ip(&req, state.config.server.trust_proxy_headers)
        ),
    };
    enforce(&state, AI_POLICY, &subject, req, next).await
}

/// Caps unauthenticated catalog and insight reads per IP. Mounted only when
/// `PUBLIC_RATE_LIMIT_PER_MINUTE` is positive.
pub async fn public_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req, state.config.server.trust_proxy_headers);
    let policy = public_policy(state.config.server.public_rate_limit_per_minute);
    enforce(&state, policy, &ip, req, next).await
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
        let forwarded = request_with_ip("x-forwarded-for", "garbage, 198.51.100.4, 10.0.0.1");
        assert_eq!(extract_client_ip(&forwarded, true), "198.51.100.4");
        let bare = Request::builder().uri("/").body(Body::empty()).unwrap();
        assert_eq!(extract_client_ip(&bare, true), "127.0.0.1");
    }

    #[test]
    fn public_policy_takes_cap_from_config() {
        let policy = public_policy(600);
        assert_eq!(policy.max_requests, 600);
        assert_eq!(policy.key_prefix, "rate_limit:public");
    }
}
