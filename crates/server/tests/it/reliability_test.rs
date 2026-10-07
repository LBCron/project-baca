//! Fault injection and graceful degradation.

use crate::common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestHarness;
use infra::{build_embedding_provider, AiConfig, AppConfig};
use mockall::automock;
use pretty_assertions::assert_eq;
use sea_orm::DatabaseConnection;
use server::{create_app, AppState};
use shared::{AppError, BookSummaryDto};
use std::sync::Arc;
use uuid::Uuid;

#[automock]
pub trait BookCatalogPort: Send + Sync {
    fn find_book_by_id(&self, id: &Uuid) -> Result<Option<BookSummaryDto>, AppError>;
}

#[tokio::test]
async fn test_reliability_disconnected_database_fallback() {
    let mut config = AppConfig::default();
    config.database.url = "postgresql://invalid_host:5433/none".to_string();
    let config = Arc::new(config);

    let redis = redis::Client::open("redis://127.0.0.1:6380").unwrap();

    let embedding = build_embedding_provider(&AiConfig {
        provider: "fastembed".to_string(),
        api_key: String::new(),
        model_name: "text-embedding-004".to_string(),
        dimension: 768,
    })
    .expect("fastembed stub must always succeed");

    let state = Arc::new(AppState {
        db: DatabaseConnection::Disconnected,
        redis,
        redis_conn: None,
        config,
        embedding,
        storage: None,
    });

    let app = create_app(state);

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let (resp, body) = {
        let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
        let (parts, body) = resp.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (
            axum::http::Response::from_parts(parts, Body::from(bytes)),
            json,
        )
    };

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["data"]["postgres"], "disconnected");
}

#[tokio::test]
async fn test_reliability_mock_repository_fault_injection() {
    let mut mock_port = MockBookCatalogPort::new();
    let sample_id = Uuid::new_v4();

    mock_port
        .expect_find_book_by_id()
        .with(mockall::predicate::eq(sample_id))
        .times(1)
        .returning(|_| {
            Err(AppError::Internal(
                "Database connection pool exhausted".to_string(),
            ))
        });

    let result = mock_port.find_book_by_id(&sample_id);

    claims::assert_err!(&result);
    match result {
        Err(AppError::Internal(msg)) => {
            assert!(msg.contains("pool exhausted"));
        }
        _ => panic!("Expected AppError::Internal"),
    }
}

#[tokio::test]
async fn test_reliability_oversized_request_id_handling() {
    let harness = TestHarness::new().await;
    let long_id = "req_".to_string() + &"a".repeat(1000);

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .header("x-request-id", long_id)
        .body(Body::empty())
        .unwrap();

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let returned = resp.headers()["x-request-id"].to_str().unwrap();
    assert!(
        Uuid::parse_str(returned).is_ok(),
        "oversized client id must be replaced by a server UUID, got {returned}"
    );
}

/// Signup fails closed when the mail relay is unreachable: no 201 without a
/// deliverable code, and the OTP cooldown must not block the retry.
#[tokio::test]
async fn test_signup_fails_closed_when_smtp_unreachable() {
    let harness = TestHarness::with_config(|c| {
        c.email.smtp_host = "127.0.0.1".to_string();
        c.email.smtp_port = 1;
    })
    .await;
    if harness.live_only().await.is_none() {
        panic!("live Postgres + Redis required (db-up)");
    }
    let email = format!("smtp_down_{}@example.com", uuid::Uuid::new_v4());
    let signup = || {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/signup")
            .header("cf-connecting-ip", "203.0.113.250")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "display_name": "Relay Down",
                    "email": email,
                    "password": "RelayDown123!"
                })
                .to_string(),
            ))
            .unwrap()
    };

    let (resp, body) = harness.send_json_request(signup()).await;
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "EXTERNAL_SERVICE_ERROR");

    // Immediate retry hits the relay again instead of the 60 s cooldown.
    let (resp, body) = harness.send_json_request(signup()).await;
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "EXTERNAL_SERVICE_ERROR");
}
