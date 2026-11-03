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
        .header(common::CLIENT_IP_HEADER, "10.0.0.1")
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
        .header(common::CLIENT_IP_HEADER, "10.0.0.2")
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
        .header(common::CLIENT_IP_HEADER, "10.0.0.3")
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
        .header(common::CLIENT_IP_HEADER, "10.0.0.4")
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

/// Search and filter strings are bounded before they reach the database:
/// a 201-character query or an oversized filter is a structured 400 that
/// names the field without echoing the value.
#[tokio::test]
async fn test_api_boundary_search_and_filter_length_caps() {
    let harness = TestHarness::new().await;
    let get = |uri: String| {
        Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    };

    let (resp, body) = harness
        .send_json_request(get(format!("/api/v1/books/search?q={}", "a".repeat(201))))
        .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
    assert!(body["error"]["details"]["q"].is_array());
    assert!(
        !body["error"]["message"].as_str().unwrap().contains("aaaa"),
        "the submitted value is not echoed back"
    );

    let (resp, body) = harness
        .send_json_request(get("/api/v1/books/search?q=a".to_string()))
        .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
    assert!(body["error"]["details"]["q"].is_array());

    let (resp, body) = harness
        .send_json_request(get(format!(
            "/api/v1/books/search?q=ab&language={}",
            "x".repeat(11)
        )))
        .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
    assert!(body["error"]["details"]["language"].is_array());

    for (filter, max) in [("tag", 50), ("theme", 50), ("language", 10)] {
        let (resp, body) = harness
            .send_json_request(get(format!(
                "/api/v1/books?{filter}={}",
                "t".repeat(max + 1)
            )))
            .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{filter}: {body}");
        assert!(body["error"]["details"][filter].is_array(), "{filter}");
    }

    let (resp, body) = harness
        .send_json_request(get(format!("/api/v1/books/search?q={}", "a".repeat(200))))
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    harness.cleanup().await;
}

/// Rejections axum raises before a handler runs (path, query, content type,
/// JSON syntax, a missing field, a wrong method) answer the same JSON
/// envelope as every handler error, keeping their status.
#[tokio::test]
async fn test_every_rejection_answers_the_json_envelope() {
    let harness = TestHarness::new().await;
    // (method, uri, content type, body, status, error code)
    type Case<'a> = (
        &'a str,
        &'a str,
        Option<&'a str>,
        &'a str,
        StatusCode,
        &'a str,
    );
    let cases: [Case; 7] = [
        (
            "GET",
            "/api/v1/books/null%2Cnull",
            None,
            "",
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
        ),
        (
            "GET",
            "/api/v1/books?limit=abc",
            None,
            "",
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
        ),
        (
            "POST",
            "/api/v1/auth/login",
            None,
            "{}",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "BAD_REQUEST",
        ),
        (
            "POST",
            "/api/v1/auth/login",
            Some("application/json"),
            "{bad",
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
        ),
        (
            "POST",
            "/api/v1/auth/login",
            Some("application/json"),
            "{}",
            StatusCode::UNPROCESSABLE_ENTITY,
            "VALIDATION_FAILED",
        ),
        (
            "POST",
            "/api/v1/auth/signup",
            Some("application/json"),
            "",
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
        ),
        (
            "PUT",
            "/api/v1/books",
            None,
            "",
            StatusCode::METHOD_NOT_ALLOWED,
            "METHOD_NOT_ALLOWED",
        ),
    ];
    for (index, (method, uri, content_type, body, status, code)) in cases.into_iter().enumerate() {
        let mut req = Request::builder()
            .method(method)
            .uri(uri)
            .header(common::CLIENT_IP_HEADER, format!("10.9.0.{index}"));
        if let Some(ct) = content_type {
            req = req.header(header::CONTENT_TYPE, ct);
        }
        let req = req.body(Body::from(body)).expect("valid request");
        let (resp, json) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), status, "{method} {uri}: {json}");
        assert_eq!(json["success"], false, "{method} {uri}: {json}");
        let types: Vec<_> = resp
            .headers()
            .get_all(header::CONTENT_TYPE)
            .iter()
            .collect();
        assert_eq!(types, ["application/json"], "{method} {uri}");
        assert_eq!(json["error"]["code"], code, "{method} {uri}: {json}");
        assert!(
            json["error"]["message"]
                .as_str()
                .is_some_and(|m| !m.is_empty()),
            "{method} {uri}: {json}"
        );
        if status == StatusCode::METHOD_NOT_ALLOWED {
            assert!(
                resp.headers().contains_key(header::ALLOW),
                "405 keeps its Allow header"
            );
        }
    }
    harness.cleanup().await;
}

/// Postgres refuses NUL in text; a NUL anywhere a reader can type answers
/// 400 instead of surfacing as a database failure.
#[tokio::test]
async fn test_a_nul_byte_in_text_is_a_bad_request() {
    let harness = TestHarness::new().await;
    for uri in [
        "/api/v1/books/search?q=ab%00cd",
        "/api/v1/books?theme=a%00b",
    ] {
        let req = Request::builder()
            .uri(uri)
            .header(common::CLIENT_IP_HEADER, "10.9.1.1")
            .body(Body::empty())
            .expect("valid request");
        let (resp, json) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{uri}: {json}");
        assert_eq!(json["error"]["code"], "BAD_REQUEST", "{uri}: {json}");
    }
    harness.cleanup().await;
}

/// The spec states what the server does: shared error answers on every
/// operation, the validators' bounds, the envelope around list bodies, and
/// a token that logout takes only optionally.
#[test]
fn test_openapi_documents_errors_bounds_and_envelopes() {
    use utoipa::OpenApi;
    let doc = serde_json::to_value(server::ApiDoc::openapi()).expect("spec serialises");
    let paths = doc["paths"].as_object().expect("paths");
    for (path, item) in paths {
        for (method, op) in item.as_object().expect("path item") {
            let responses = op["responses"].as_object().expect("responses");
            let mut wanted = vec!["400", "429", "500"];
            if op.get("requestBody").is_some() {
                wanted.extend(["415", "422"]);
            }
            for status in wanted {
                assert!(
                    responses.contains_key(status),
                    "{method} {path} lacks {status}"
                );
            }
        }
    }
    let search = &doc["paths"]["/api/v1/books/search"]["get"]["parameters"];
    let q = search
        .as_array()
        .and_then(|ps| ps.iter().find(|p| p["name"] == "q"))
        .expect("q parameter");
    assert_eq!(q["schema"]["minLength"], 2);
    assert_eq!(q["schema"]["maxLength"], 200);
    for (path, method) in [("/api/v1/me/sessions", "get"), ("/api/v1/quotes", "get")] {
        let body = doc["paths"][path][method]["responses"]["200"].to_string();
        assert!(
            body.contains("ApiResponse") || body.contains("success"),
            "{path}: {body}"
        );
    }
    let logout = &doc["paths"]["/api/v1/auth/logout"]["post"]["security"];
    assert_eq!(logout.as_array().map(Vec::len), Some(2), "{logout}");
    assert!(!doc["paths"]["/api/v1/auth/logout"]["post"]["responses"]
        .as_object()
        .expect("responses")
        .contains_key("401"));
}
