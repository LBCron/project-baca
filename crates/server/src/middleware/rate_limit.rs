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
use infra::{
    hit_window, is_token_blacklisted, is_user_token_revoked, verify_access_token, Claims,
    ServerConfig,
};
use shared::{ApiResponse, AppError};
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

/// Window policy: how many requests a subject may make per window and how
/// the refusal is reported.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitPolicy {
    pub key_prefix: &'static str,
    pub max_requests: i64,
    pub window_seconds: u64,
    pub error_code: &'static str,
    pub subject_label: &'static str,
    pub window_label: &'static str,
}

/// Public auth endpoints: 20 req/min per IP.
pub const AUTH_POLICY: RateLimitPolicy = RateLimitPolicy {
    key_prefix: "rate_limit:auth",
    max_requests: 20,
    window_seconds: 60,
    error_code: "RATE_LIMITED",
    subject_label: "requests",
    window_label: "minute",
};

/// Embedding-backed search: 10 req/min per user, or per IP for guests.
pub const AI_POLICY: RateLimitPolicy = RateLimitPolicy {
    key_prefix: "rate_limit:ai",
    max_requests: 10,
    window_seconds: 60,
    error_code: "AI_RATE_LIMITED",
    subject_label: "semantic search requests",
    window_label: "minute",
};

/// Writes of one signed-in account per minute: room for page turns saved
/// every 1.2 s on two devices, far below what fills storage by script.
pub const USER_WRITE_POLICY: RateLimitPolicy = RateLimitPolicy {
    key_prefix: "rl:user",
    max_requests: 120,
    window_seconds: 60,
    error_code: "RATE_LIMITED",
    subject_label: "writes",
    window_label: "minute",
};

/// Embedding-backed requests of every reader together per day, so abuse
/// that slips past the per-reader limit still has a price ceiling.
fn ai_budget_policy(max_requests: u32) -> RateLimitPolicy {
    RateLimitPolicy {
        key_prefix: "budget:ai",
        max_requests: i64::from(max_requests),
        window_seconds: 86_400,
        error_code: "AI_RATE_LIMITED",
        subject_label: "semantic search requests",
        window_label: "day",
    }
}

/// Unauthenticated catalog and insight reads; the cap comes from config.
pub fn public_policy(max_requests: u32) -> RateLimitPolicy {
    RateLimitPolicy {
        key_prefix: "rate_limit:public",
        max_requests: i64::from(max_requests),
        window_seconds: 60,
        error_code: "RATE_LIMITED",
        subject_label: "requests",
        window_label: "minute",
    }
}

/// Address every direct client shares when no trusted header is available.
const FALLBACK_CLIENT_IP: &str = "127.0.0.1";

/// Client IP from headers. Only the one header named by
/// `TRUSTED_IP_HEADER` is consulted, and only when the deployment sets
/// `TRUST_PROXY_HEADERS=true` because its edge overwrites that header. For a
/// list-valued header such as `x-forwarded-for` the first parseable hop is
/// the client, since a proxy without upstream trust replaces the list.
/// Any other address header is ignored, so a client cannot choose its bucket.
pub fn client_ip_from_headers(headers: &HeaderMap, server: &ServerConfig) -> String {
    if !server.trust_proxy_headers {
        return FALLBACK_CLIENT_IP.to_string();
    }
    headers
        .get(server.trusted_ip_header.as_str())
        .and_then(|v| v.to_str().ok())
        .and_then(|value| {
            value
                .split(',')
                .map(str::trim)
                .find(|hop| hop.parse::<IpAddr>().is_ok())
        })
        .unwrap_or(FALLBACK_CLIENT_IP)
        .to_string()
}

/// The rate-limit subject of a client address. An IPv6 address is cut to
/// its /64, the block one customer holds, so rotating addresses inside it
/// buys nothing; an IPv4-mapped IPv6 address counts as its IPv4.
pub fn rate_subject(ip: &str) -> String {
    match ip.parse::<IpAddr>() {
        Ok(IpAddr::V6(v6)) => match v6.to_ipv4_mapped() {
            Some(v4) => v4.to_string(),
            None => {
                let s = v6.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        },
        Ok(IpAddr::V4(v4)) => v4.to_string(),
        Err(_) => ip.to_string(),
    }
}

/// Rate-limit subject of the client behind `headers`; every per-client
/// limit and lockout keys on this, never on the raw address.
pub fn client_subject_from_headers(headers: &HeaderMap, server: &ServerConfig) -> String {
    rate_subject(&client_ip_from_headers(headers, server))
}

/// Client IP for rate limiting (see [`client_ip_from_headers`]).
pub fn extract_client_ip(req: &Request<Body>, server: &ServerConfig) -> String {
    client_ip_from_headers(req.headers(), server)
}

fn extract_client_subject(req: &Request<Body>, server: &ServerConfig) -> String {
    client_subject_from_headers(req.headers(), server)
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

/// Counts one hit for `subject`: the count and window TTL, or the response
/// that refuses the request. Fails closed when Redis is unreachable: an
/// unenforceable limit must shed traffic, not open it.
async fn count_hit(
    state: &AppState,
    policy: RateLimitPolicy,
    subject: &str,
) -> Result<(i64, i64), Box<Response>> {
    let unavailable =
        || Box::new(HttpError(AppError::ServiceUnavailable { retry_after: 60 }).into_response());
    let redis_key = format!("{}:{subject}", policy.key_prefix);
    let mut redis_conn = state.get_redis_conn().await.map_err(|_| unavailable())?;
    let (count, ttl) = hit_window(&mut redis_conn, &redis_key, policy.window_seconds)
        .await
        .map_err(|_| unavailable())?;
    let ttl = ttl.max(1);
    if count <= policy.max_requests {
        return Ok((count, ttl));
    }
    let mut resp = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(ApiResponse::<()>::error(
            policy.error_code,
            &format!(
                "Rate limit exceeded. Maximum {} {} per {} allowed.",
                policy.max_requests, policy.subject_label, policy.window_label
            ),
            None,
        )),
    )
        .into_response();
    set_limit_headers(resp.headers_mut(), policy.max_requests, 0, ttl);
    if let Ok(v) = HeaderValue::from_str(&ttl.to_string()) {
        resp.headers_mut().insert(header::RETRY_AFTER, v);
    }
    Err(Box::new(resp))
}

/// Counts one hit against every limit and either forwards the request, with
/// RFC rate-limit headers for the first limit, or answers with the refusal
/// of the first limit that is spent.
async fn enforce(
    state: &AppState,
    limits: &[(RateLimitPolicy, String)],
    req: Request<Body>,
    next: Next,
) -> Response {
    let mut headline = None;
    for (policy, subject) in limits {
        match count_hit(state, *policy, subject).await {
            Ok((count, ttl)) => {
                headline.get_or_insert((policy.max_requests, count, ttl));
            }
            Err(refusal) => return *refusal,
        }
    }
    let mut response = next.run(req).await;
    if let Some((limit, count, ttl)) = headline {
        set_limit_headers(response.headers_mut(), limit, (limit - count).max(0), ttl);
    }
    response
}

/// Caps public auth endpoints per IP.
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let subject = extract_client_subject(&req, &state.config.server);
    enforce(&state, &[(AUTH_POLICY, subject)], req, next).await
}

/// Caps embedding-backed search: signed-in users by id, guests by IP. A
/// revoked token buys no per-user bucket; it counts as a guest.
pub async fn ai_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let subject = match bearer_user(&state, req.headers()).await {
        Some(id) => format!("user:{id}"),
        None => format!(
            "guest:{}",
            extract_client_subject(&req, &state.config.server)
        ),
    };
    let mut limits = vec![(AI_POLICY, subject)];
    let budget = state.config.ai.daily_budget;
    if budget > 0 {
        limits.push((ai_budget_policy(budget), "all".to_string()));
    }
    enforce(&state, &limits, req, next).await
}

/// Caps writes per account on bearer routes. Reads pass, and so does a
/// request without a live token: its handler answers 401 before writing.
pub async fn user_write_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if req.method().is_safe() {
        return next.run(req).await;
    }
    match bearer_user(&state, req.headers()).await {
        Some(id) => enforce(&state, &[(USER_WRITE_POLICY, id.to_string())], req, next).await,
        None => next.run(req).await,
    }
}

/// The account behind a live bearer token, if the request carries one.
async fn bearer_user(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    let claims = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| verify_access_token(token, state.config.jwt_secret()).ok())?;
    token_still_live(state, &claims).await.then_some(claims.sub)
}

/// False when the token was blacklisted or its user revoked after issue.
/// A Redis error also answers false; `enforce` then fails closed on its own.
async fn token_still_live(state: &AppState, claims: &Claims) -> bool {
    let Ok(mut redis) = state.get_redis_conn().await else {
        return false;
    };
    matches!(
        is_token_blacklisted(&mut redis, claims.jti).await,
        Ok(false)
    ) && matches!(
        is_user_token_revoked(&mut redis, claims.sub, claims.iat).await,
        Ok(false)
    )
}

/// Caps unauthenticated catalog and insight reads per IP. Mounted only when
/// `PUBLIC_RATE_LIMIT_PER_MINUTE` is positive.
pub async fn public_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let subject = extract_client_subject(&req, &state.config.server);
    let policy = public_policy(state.config.server.public_rate_limit_per_minute);
    enforce(&state, &[(policy, subject)], req, next).await
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

    fn trusting(header: &str) -> ServerConfig {
        ServerConfig {
            trust_proxy_headers: true,
            trusted_ip_header: header.to_string(),
            ..ServerConfig::default()
        }
    }

    #[test]
    fn one_ipv6_block_is_one_subject() {
        assert_eq!(
            rate_subject("2001:db8::1"),
            rate_subject("2001:db8::ffff:2")
        );
        assert_eq!(rate_subject("2001:db8::1"), "2001:db8:0:0::/64");
        assert_ne!(
            rate_subject("2001:db8:0:1::1"),
            rate_subject("2001:db8:0:2::1")
        );
        assert_eq!(rate_subject("::ffff:203.0.113.7"), "203.0.113.7");
        assert_eq!(rate_subject("203.0.113.7"), "203.0.113.7");
    }

    #[test]
    fn untrusted_mode_ignores_spoofed_headers() {
        let req = request_with_ip("cf-connecting-ip", "203.0.113.7");
        assert_eq!(
            extract_client_ip(&req, &ServerConfig::default()),
            FALLBACK_CLIENT_IP
        );
    }

    #[test]
    fn trusted_mode_reads_only_the_configured_header() {
        let edge = trusting("cf-connecting-ip");
        let req = request_with_ip("cf-connecting-ip", "203.0.113.7");
        assert_eq!(extract_client_ip(&req, &edge), "203.0.113.7");

        let spoofed = Request::builder()
            .uri("/")
            .header("x-real-ip", "203.0.113.9")
            .header("x-forwarded-for", "203.0.113.10")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_ip(&spoofed, &edge), FALLBACK_CLIENT_IP);

        let bare = Request::builder().uri("/").body(Body::empty()).unwrap();
        assert_eq!(extract_client_ip(&bare, &edge), FALLBACK_CLIENT_IP);
    }

    #[test]
    fn forwarded_for_takes_the_first_parseable_hop() {
        let edge = trusting("x-forwarded-for");
        let forwarded = request_with_ip("x-forwarded-for", "garbage, 198.51.100.4, 10.0.0.1");
        assert_eq!(extract_client_ip(&forwarded, &edge), "198.51.100.4");
        let real_ip_only = request_with_ip("x-real-ip", "198.51.100.4");
        assert_eq!(extract_client_ip(&real_ip_only, &edge), FALLBACK_CLIENT_IP);
    }

    #[test]
    fn public_policy_takes_cap_from_config() {
        let policy = public_policy(600);
        assert_eq!(policy.max_requests, 600);
        assert_eq!(policy.key_prefix, "rate_limit:public");
    }
}
