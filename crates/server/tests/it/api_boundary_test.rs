//! HTTP edge cases, error envelopes, and pagination bounds.

use crate::common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::TestHarness;

#[tokio::test]
async fn test_api_boundary_route_not_found_returns_standard_envelope() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/nonexistent-service/endpoint")
        .body(Body::empty())
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert!(body["error"]["message"].is_string());
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_method_not_allowed() {
    let harness = TestHarness::new().await;

    // Send POST to /health (which only permits GET)
    let req = Request::builder()
        .method("POST")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = harness.send_request(req).await;
    assert_eq!(
        resp.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "POST on /health should return 405 Method Not Allowed"
    );

    // Send PUT to /api/v1/books (which only permits GET)
    let req2 = Request::builder()
        .method("PUT")
        .uri("/api/v1/books")
        .body(Body::empty())
        .unwrap();

    let resp2 = harness.send_request(req2).await;
    assert_eq!(
        resp2.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "PUT on /api/v1/books should return 405 Method Not Allowed"
    );
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_malformed_json_payload_rejection() {
    let harness = TestHarness::new().await;

    // Intentionally broken JSON syntax (missing closing quote and brace)
    let broken_json = r#"{"email": "corrupt@input.com", "password": "#;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", "10.0.0.1")
        .header("content-type", "application/json")
        .body(Body::from(broken_json))
        .unwrap();

    let resp = harness.send_request(req).await;
    // Axum Json extractor rejects malformed syntax with 400 or 422
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Malformed JSON must be rejected with 400 or 422, received: {}",
        resp.status()
    );
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_empty_body_on_json_endpoint() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", "10.0.0.2")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let resp = harness.send_request(req).await;
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Empty body on required JSON endpoint must be rejected"
    );
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_pagination_extremes() {
    let harness = TestHarness::new().await;

    // Limit zero: should clamp to min or return valid response
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/books?limit=0")
        .body(Body::empty())
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["success"], true);

    // Limit 1000: should clamp to max (50) and return valid response
    let req2 = Request::builder()
        .method("GET")
        .uri("/api/v1/books?limit=1000")
        .body(Body::empty())
        .unwrap();

    let (resp2, body2) = harness.send_json_request(req2).await;
    assert_eq!(resp2.status(), StatusCode::OK);
    assert_eq!(body2["success"], true);

    // Invalid cursor: should not panic
    let req3 = Request::builder()
        .method("GET")
        .uri("/api/v1/books?cursor=not-a-valid-uuid-here")
        .body(Body::empty())
        .unwrap();

    let resp3 = harness.send_request(req3).await;
    // Axum Query extractor rejects invalid UUID parameter with 400 Bad Request
    assert_eq!(resp3.status(), StatusCode::BAD_REQUEST);
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_field_length_overflow_validation() {
    let harness = TestHarness::new().await;

    // display_name max length is 100 characters; sending 120 characters
    let long_display_name = "A".repeat(120);
    let payload = serde_json::json!({
        "display_name": long_display_name,
        "email": "valid_email@example.com",
        "password": "ValidPassword123!"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", "10.0.0.3")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
    assert!(
        body["error"]["details"]["display_name"].is_array(),
        "Validation error must pinpoint display_name field"
    );
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_password_min_length_validation() {
    let harness = TestHarness::new().await;

    // Password min length is 8 characters; sending 4 characters
    let payload = serde_json::json!({
        "display_name": "Short Pass User",
        "email": "short_pass@example.com",
        "password": "123"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", "10.0.0.4")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
    assert!(
        body["error"]["details"]["password"].is_array(),
        "Validation error must pinpoint password field"
    );
    harness.cleanup().await;
}

#[tokio::test]
async fn test_api_boundary_authorization_header_schemes() {
    let harness = TestHarness::new().await;

    // Completely missing Authorization header
    let req1 = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .body(Body::empty())
        .unwrap();

    let (resp1, body1) = harness.send_json_request(req1).await;
    assert_eq!(resp1.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body1["error"]["code"], "UNAUTHORIZED");

    // Basic auth scheme instead of Bearer
    let req2 = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header(header::AUTHORIZATION, "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();

    let (resp2, body2) = harness.send_json_request(req2).await;
    assert_eq!(resp2.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body2["error"]["code"], "UNAUTHORIZED");
    assert!(
        body2["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("Bearer expected"),
        "Should inform client that Bearer is required"
    );
    harness.cleanup().await;
}
