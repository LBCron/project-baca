//! Boot, health, Swagger UI, and infrastructure reachability.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestHarness;
use pretty_assertions::assert_eq;
use std::net::TcpStream;
use std::time::Duration;

#[tokio::test]
async fn test_smoke_health_endpoint() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .expect("Failed to build request");

    let (resp, body) = harness.send_json_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "healthy");
    assert_eq!(body["data"]["service"], "project-baca-server");
    assert!(resp.headers().contains_key("x-request-id"));
}

#[tokio::test]
async fn test_smoke_api_v1_health_dependencies() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .expect("Failed to build request");

    let (resp, body) = harness.send_json_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "ok");
    assert_eq!(body["data"]["redis"], "configured");
    assert!(body["data"]["postgres"] == "connected" || body["data"]["postgres"] == "disconnected");
}

#[tokio::test]
async fn test_smoke_swagger_ui_endpoint() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/swagger-ui/")
        .body(Body::empty())
        .expect("Failed to build request");

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("text/html"));
}

#[tokio::test]
async fn test_smoke_openapi_json_spec() {
    let harness = TestHarness::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api-docs/openapi.json")
        .body(Body::empty())
        .expect("Failed to build request");

    let (resp, spec) = harness.send_json_request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(spec["info"]["title"], "Project Baca REST API");
    assert_eq!(spec["info"]["version"], "0.1.0");
    assert!(spec["paths"]["/health"].is_object());
    assert!(spec["paths"]["/api/v1/health"].is_object());
}

#[test]
fn test_smoke_infrastructure_tcp_reachability() {
    let postgres_addr = "127.0.0.1:5433";
    if let Ok(_stream) =
        TcpStream::connect_timeout(&postgres_addr.parse().unwrap(), Duration::from_millis(500))
    {
        println!("PostgreSQL port 5433 is reachable.");
    }

    let redis_addr = "127.0.0.1:6380";
    if let Ok(_stream) =
        TcpStream::connect_timeout(&redis_addr.parse().unwrap(), Duration::from_millis(500))
    {
        println!("Redis port 6380 is reachable.");
    }
}
