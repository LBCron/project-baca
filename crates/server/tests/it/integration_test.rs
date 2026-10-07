//! Request cycles, request IDs, CORS, and OpenAPI schemas.

use crate::common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestHarness;
use pretty_assertions::assert_eq;
use uuid::Uuid;

#[tokio::test]
async fn test_integration_request_id_server_generated() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .header("x-request-id", "req_custom_trace_987654")
        .body(Body::empty())
        .expect("Failed to build request");

    let resp = harness.send_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let returned_id = resp
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        Uuid::parse_str(returned_id).is_ok(),
        "server must mint its own request id, got {returned_id}"
    );
}

#[tokio::test]
async fn test_integration_request_id_auto_generation() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .expect("Failed to build request");

    let resp = harness.send_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let generated_id = resp
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(Uuid::parse_str(generated_id).is_ok());
}

#[tokio::test]
async fn test_integration_cors_headers() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("OPTIONS")
        .uri("/health")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "GET")
        .body(Body::empty())
        .expect("Failed to build request");

    let resp = harness.send_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_integration_openapi_schema_contains_registered_dtos() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api-docs/openapi.json")
        .body(Body::empty())
        .expect("Failed to build request");

    let (resp, spec) = harness.send_json_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let schemas = &spec["components"]["schemas"];
    assert!(
        schemas["SignupRequest"].is_object(),
        "SignupRequest schema missing"
    );
    assert!(
        schemas["LoginRequest"].is_object(),
        "LoginRequest schema missing"
    );
    assert!(
        schemas["VerifyOtpRequest"].is_object(),
        "VerifyOtpRequest schema missing"
    );
    assert!(
        schemas["TokenResponse"].is_object(),
        "TokenResponse schema missing"
    );
    assert!(
        schemas["UserProfileDto"].is_object(),
        "UserProfileDto schema missing"
    );
    assert!(
        schemas["BookSummaryDto"].is_object(),
        "BookSummaryDto schema missing"
    );
    assert!(
        schemas["BookDetailDto"].is_object(),
        "BookDetailDto schema missing"
    );
    assert!(
        schemas["ChapterSummaryDto"].is_object(),
        "ChapterSummaryDto schema missing"
    );
    assert!(
        schemas["QuoteSearchRequest"].is_object(),
        "QuoteSearchRequest schema missing"
    );
    assert!(
        schemas["QuoteSearchResultDto"].is_object(),
        "QuoteSearchResultDto schema missing"
    );
    assert!(
        schemas["ReadingProgressUpdateDto"].is_object(),
        "ReadingProgressUpdateDto schema missing"
    );
}

#[tokio::test]
async fn test_integration_route_not_found_handling() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/non_existent_resource")
        .body(Body::empty())
        .expect("Failed to build request");

    let resp = harness.send_request(req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert!(resp.headers().contains_key("x-request-id"));
}
