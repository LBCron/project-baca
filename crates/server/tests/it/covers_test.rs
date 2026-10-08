//! Cover image delivery from the covers bucket (needs live MinIO).

use crate::common;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use common::TestHarness;
use uuid::Uuid;

fn get_cover(file: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/api/v1/covers/{file}"))
        .body(Body::empty())
        .unwrap()
}

/// A cover the worker stored is served back with its media type, a one-year
/// immutable cache policy, and a cross-origin resource policy so the reader
/// can embed it from another origin in development.
#[tokio::test]
async fn test_cover_roundtrip_through_storage() {
    let harness = TestHarness::new().await;
    let storage = harness
        .state
        .storage
        .clone()
        .expect("live MinIO required (db-up)");
    let book_id = Uuid::new_v4();
    let key = format!("covers/{book_id}.webp");
    let bytes = b"RIFF\x1a\x00\x00\x00WEBPVP8 ".to_vec();
    storage
        .put_cover(&key, bytes.clone(), "image/webp")
        .await
        .expect("cover upload must succeed");

    let resp = harness
        .send_request(get_cover(&format!("{book_id}.webp")))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/webp"
    );
    assert_eq!(
        resp.headers().get(header::CACHE_CONTROL).unwrap(),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(
        resp.headers().get("cross-origin-resource-policy").unwrap(),
        "cross-origin"
    );
    let served = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(served.as_ref(), bytes.as_slice());

    storage
        .delete_cover(&key)
        .await
        .expect("cover cleanup must succeed");
    harness.cleanup().await;
}

/// Unknown covers and names outside `<uuid>.<image ext>` both answer 404
/// with the standard envelope; no other bucket key is reachable.
#[tokio::test]
async fn test_cover_missing_and_invalid_names_are_404() {
    let harness = TestHarness::new().await;
    harness
        .state
        .storage
        .as_ref()
        .expect("live MinIO required (db-up)");

    let (resp, body) = harness
        .send_json_request(get_cover(&format!("{}.webp", Uuid::new_v4())))
        .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "NOT_FOUND");

    for name in [
        "cover.txt",
        "..%2Fraw-epubs%2Fx.epub",
        "a0000000-0000-4000-8000-000000000001.svg",
        "A0000000-0000-4000-8000-000000000001.webp",
        "not-a-uuid.png",
    ] {
        let (resp, body) = harness.send_json_request(get_cover(name)).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{name}: {body}");
        assert_eq!(body["error"]["code"], "NOT_FOUND", "{name}");
    }
    harness.cleanup().await;
}
