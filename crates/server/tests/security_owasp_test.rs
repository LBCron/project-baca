//! OWASP ASVS: error handling, throttling, and revocation.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::TestHarness;
use pretty_assertions::assert_eq;
use redis::AsyncCommands;
use shared::{ChangePasswordRequest, LoginRequest, RefreshTokenRequest, SignupRequest};
use uuid::Uuid;

fn unique_test_ip() -> String {
    let u = Uuid::new_v4().as_u128();
    format!("198.51.{}.{}", ((u >> 8) % 250) + 1, (u % 250) + 1)
}

#[tokio::test]
async fn test_owasp_security_headers_and_error_handling() {
    let harness = TestHarness::new().await;

    // Verify standard OWASP security headers on any public route
    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
    assert_eq!(
        resp.headers().get("referrer-policy").unwrap(),
        "strict-origin-when-cross-origin"
    );
    assert_eq!(
        resp.headers().get("content-security-policy").unwrap(),
        "default-src 'none'; frame-ancestors 'none'; base-uri 'none'"
    );
    assert_eq!(
        resp.headers().get("cross-origin-opener-policy").unwrap(),
        "same-origin"
    );
    assert_eq!(
        resp.headers().get("cross-origin-resource-policy").unwrap(),
        "same-origin"
    );

    // Swagger UI needs inline scripts/styles, so its document CSP is relaxed
    // to same-origin + unsafe-inline while every other route stays strict.
    let req = Request::builder()
        .method("GET")
        .uri("/swagger-ui/")
        .body(Body::empty())
        .unwrap();
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let swagger_csp = resp
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        swagger_csp.contains("'unsafe-inline'"),
        "swagger CSP must allow its bundled inline scripts, got {swagger_csp}"
    );

    // Structured validation errors on bad signup payload (CWE-209 prevention)
    let invalid_signup = serde_json::json!({
        "email": "invalid-email-format",
        "password": "short",
        "display_name": "A"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", unique_test_ip())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_signup).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
    assert!(body["error"]["details"].is_object());
    assert!(body["error"]["details"]["email"].is_array());
    assert!(body["error"]["details"]["password"].is_array());
}

#[tokio::test]
async fn test_owasp_rate_limiting_headers_and_ip_extraction() {
    let harness = TestHarness::new().await;
    let client_ip = unique_test_ip();

    // Send request with CF-Connecting-IP
    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .header("cf-connecting-ip", &client_ip)
        .body(Body::empty())
        .unwrap();

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Request through auth router has rate limit headers
    let dummy_login = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "wrongpassword123".to_string(),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", &client_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&dummy_login).unwrap()))
        .unwrap();

    let resp = harness.send_request(req).await;
    assert!(resp.headers().contains_key("x-ratelimit-limit"));
    assert!(resp.headers().contains_key("x-ratelimit-remaining"));
    assert!(resp.headers().contains_key("x-ratelimit-reset"));
}

#[tokio::test]
async fn test_owasp_otp_cooldown_and_email_bombing_prevention() {
    let harness = TestHarness::new().await;
    let _ = harness
        .live_only()
        .await
        .expect("live Redis required (db-up); 11c gate");

    let target_email = format!("victim_{}@example.com", Uuid::new_v4());
    let signup_req = SignupRequest {
        display_name: "TargetUser".to_string(),
        email: target_email.clone(),
        password: "ValidPassword123!".to_string(),
    };
    let client_ip = unique_test_ip();

    // First signup: Success (201 Created)
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", &client_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(body["success"], true);

    // Immediate second signup for the same email: Throttled (429 Too Many Requests)
    let req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", &client_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();

    let (resp2, body2) = harness.send_json_request(req2).await;
    assert_eq!(resp2.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body2["error"]["code"], "RATE_LIMITED");
    assert!(resp2.headers().contains_key(header::RETRY_AFTER));
}

#[tokio::test]
async fn test_owasp_login_brute_force_lockout() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up); 11c gate");

    let target_email = format!("targeted_{}@example.com", Uuid::new_v4());
    let bad_login = LoginRequest {
        email: target_email.clone(),
        password: "WrongPassword123!".to_string(),
    };
    let client_ip = unique_test_ip();

    // Send 5 incorrect login attempts
    for _ in 0..5 {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("cf-connecting-ip", &client_ip)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&bad_login).unwrap()))
            .unwrap();
        let (resp, _) = harness.send_json_request(req).await;
        assert!(
            resp.status() == StatusCode::UNAUTHORIZED
                || resp.status() == StatusCode::TOO_MANY_REQUESTS
        );
    }

    // 6th attempt MUST be locked out with 429 Too Many Requests
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", &client_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&bad_login).unwrap()))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "RATE_LIMITED");
    assert!(resp.headers().contains_key(header::RETRY_AFTER));

    // Cleanup lockout keys in redis (pair + aggregate, see note_login_failure)
    let pair_fail = format!("auth:login:fail:{target_email}:{client_ip}");
    let pair_lock = format!("auth:login:lockout:{target_email}:{client_ip}");
    let lockout_key = format!("auth:login:lockout:{target_email}");
    let fail_key = format!("auth:login:fail:{target_email}");
    let _: Result<(), _> = redis_conn
        .del(&[&pair_fail, &pair_lock, &lockout_key, &fail_key])
        .await;
}

#[tokio::test]
async fn test_owasp_password_change_identical_rejection() {
    let harness = TestHarness::new().await;

    // Same-value password change must return 400.
    let change_pwd_req = ChangePasswordRequest {
        current_password: "CurrentPassword123!".to_string(),
        new_password: "CurrentPassword123!".to_string(),
        revoke_other_sessions: None,
    };

    // Create a dummy token for AuthUser
    let token = infra::generate_access_token(
        Uuid::new_v4(),
        "user@test.local",
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .unwrap();

    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/password")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&change_pwd_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("different from current password"));
}

#[tokio::test]
async fn test_owasp_token_revocation_on_logout() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up); 11c gate");

    let test_user_id = Uuid::new_v4();
    let access_token = infra::generate_access_token(
        test_user_id,
        "token_revoker@test.local",
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .unwrap();

    let refresh_token = infra::generate_refresh_token();
    infra::store_refresh_token(&mut redis_conn, &refresh_token, test_user_id, 14)
        .await
        .unwrap();

    // Verify access token works initially (get profile might 404 on DB, but NOT 401)
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Logout providing both refresh token and Bearer access token
    let logout_req = RefreshTokenRequest {
        refresh_token: refresh_token.clone(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("cf-connecting-ip", unique_test_ip())
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&logout_req).unwrap()))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["success"], true);

    // Subsequent request with blacklisted access token MUST return 401 UNAUTHORIZED
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("revoked"));
}

#[tokio::test]
async fn test_owasp_revoke_all_sessions() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up); 11c gate");

    let test_user_id = Uuid::new_v4();
    let access_token = infra::generate_access_token(
        test_user_id,
        "multi_session@test.local",
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .unwrap();

    let refresh_token1 = infra::generate_refresh_token();
    let refresh_token2 = infra::generate_refresh_token();
    infra::store_refresh_token(&mut redis_conn, &refresh_token1, test_user_id, 14)
        .await
        .unwrap();
    infra::store_refresh_token(&mut redis_conn, &refresh_token2, test_user_id, 14)
        .await
        .unwrap();

    // Call /api/v1/auth/revoke-all
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/revoke-all")
        .header("cf-connecting-ip", unique_test_ip())
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["success"], true);

    // Refresh token 1 should be gone from Redis
    let exists1: bool = redis_conn
        .exists(format!("refresh_token:{refresh_token1}"))
        .await
        .unwrap();
    assert!(!exists1);

    // Refresh token 2 should be gone from Redis
    let exists2: bool = redis_conn
        .exists(format!("refresh_token:{refresh_token2}"))
        .await
        .unwrap();
    assert!(!exists2);

    // Access token issued before revocation timestamp should now be rejected
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_owasp_account_deletion_session_cleanup() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up); 11c gate");

    let test_user_id = Uuid::new_v4();
    let access_token = infra::generate_access_token(
        test_user_id,
        "deleter@test.local",
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .unwrap();

    let refresh_token = infra::generate_refresh_token();
    infra::store_refresh_token(&mut redis_conn, &refresh_token, test_user_id, 14)
        .await
        .unwrap();

    // Delete account
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["success"], true);

    // Refresh token must be revoked from Redis
    let exists: bool = redis_conn
        .exists(format!("refresh_token:{refresh_token}"))
        .await
        .unwrap();
    assert!(
        !exists,
        "Refresh token should be revoked after account deletion"
    );

    // Access token must be rejected immediately on subsequent requests
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_owasp_timing_attack_mitigation_on_login() {
    let harness = TestHarness::new().await;
    let _ = harness
        .live_only()
        .await
        .expect("live Redis required (db-up); 11c gate");

    let non_existent_email = format!("ghost_{}@example.com", Uuid::new_v4());
    let login_req = LoginRequest {
        email: non_existent_email,
        password: "WrongPassword123!".to_string(),
    };

    let client_ip = unique_test_ip();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", &client_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
    assert_eq!(body["error"]["message"], "Invalid email or password");
}

#[tokio::test]
async fn test_owasp_structured_validation_error_details() {
    let harness = TestHarness::new().await;

    // Bad signup payload: invalid email format and short password
    let bad_req = serde_json::json!({
        "display_name": "",
        "email": "not-an-email",
        "password": "short"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", &unique_test_ip())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&bad_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
    assert!(body["error"]["details"].is_object());
    assert!(body["error"]["details"]["email"].is_array());
}
