//! Concurrent throughput, pool resilience, and rate-limit shedding.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use common::TestHarness;
use infra::entities::users;
use sea_orm::{ActiveModelTrait, Set};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

#[tokio::test]
async fn test_load_concurrent_catalog_traffic_200_tasks() {
    let harness = Arc::new(TestHarness::new().await);
    let task_count = 200;
    let mut handles = Vec::with_capacity(task_count);

    let start = Instant::now();

    for i in 0..task_count {
        let h = Arc::clone(&harness);
        handles.push(tokio::spawn(async move {
            // Alternate between catalog listing and health check
            let uri = if i % 2 == 0 {
                "/api/v1/books?limit=5"
            } else {
                "/health"
            };

            let req = Request::builder()
                .method("GET")
                .uri(uri)
                .header("x-request-id", format!("load_task_{i}"))
                .body(Body::empty())
                .unwrap();

            let resp = h.send_request(req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for handle in handles {
        handle.await.expect("Concurrent load task panicked");
    }

    let elapsed = start.elapsed();
    let rps = (task_count as f64) / elapsed.as_secs_f64();
    println!(
        "Load test (200 tasks): completed in {} ms ({:.1} req/sec)",
        elapsed.as_millis(),
        rps
    );

    // 200 tasks in-memory should easily complete in under 2 seconds
    assert!(
        elapsed.as_millis() < 2000,
        "200 concurrent tasks exceeded 2000ms SLA: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_load_concurrent_authenticated_profile_reads() {
    let harness = Arc::new(TestHarness::new().await);
    let user_id = Uuid::new_v4();
    let test_email = format!("load_auth_{}@example.com", user_id);
    let now = Utc::now();

    // Seed active user in database
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(test_email.clone()),
        password_hash: Set(Some("hash".to_string())),
        display_name: Set("Load Auth User".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    let _ = user.insert(&harness.state.db).await;

    // Issue valid access token
    let token = infra::generate_access_token(
        user_id,
        &test_email,
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .expect("Token generation failed");

    // Fire 100 concurrent authenticated requests
    let task_count = 100;
    let mut handles = Vec::with_capacity(task_count);
    let start = Instant::now();

    for i in 0..task_count {
        let h = Arc::clone(&harness);
        let t = token.clone();
        handles.push(tokio::spawn(async move {
            let req = Request::builder()
                .method("GET")
                .uri("/api/v1/me")
                .header("authorization", format!("Bearer {t}"))
                .header("x-request-id", format!("auth_load_{i}"))
                .body(Body::empty())
                .unwrap();

            let resp = h.send_request(req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for handle in handles {
        handle.await.expect("Authenticated load worker panicked");
    }

    let elapsed = start.elapsed();
    println!(
        "Authenticated load test (100 workers): completed in {} ms ({:.1} req/sec)",
        elapsed.as_millis(),
        (task_count as f64) / elapsed.as_secs_f64()
    );

    assert!(
        elapsed.as_millis() < 1500,
        "100 authenticated requests exceeded 1500ms SLA"
    );
}

#[tokio::test]
async fn test_stress_rate_limit_saturation_and_shedding() {
    let harness = TestHarness::new().await;
    let _ = harness
        .live_only()
        .await
        .expect("live Redis required (db-up)");

    // A single IP sends 50 requests in rapid succession to /api/v1/auth/login
    // The rate limiter limit is 20 req/minute
    let spammer_ip = format!("198.51.100.{}", (Uuid::new_v4().as_u128() % 250) + 1);
    let total_requests = 40;
    let mut accepted_count = 0;
    let mut rejected_count = 0;

    let start = Instant::now();

    for i in 0..total_requests {
        let login_body = serde_json::json!({
            "email": format!("spammer_{i}@attack.local"),
            "password": "WrongPassword123!"
        });
        let payload_bytes = serde_json::to_vec(&login_body).unwrap();

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header(common::CLIENT_IP_HEADER, &spammer_ip)
            .header("content-type", "application/json")
            .body(Body::from(payload_bytes))
            .unwrap();

        let (resp, body) = harness.send_json_request(req).await;

        if resp.status() == StatusCode::UNAUTHORIZED {
            accepted_count += 1;
        } else if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            rejected_count += 1;
            assert_eq!(body["error"]["code"], "RATE_LIMITED");
            assert!(
                resp.headers().contains_key(header::RETRY_AFTER),
                "429 responses must include Retry-After header"
            );
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Stress test (40 rapid requests from single IP): processed: {accepted_count}, shed: {rejected_count} in {} ms",
        elapsed.as_millis()
    );

    assert_eq!(
        accepted_count, 20,
        "Exactly 20 requests should be accepted before limit"
    );
    assert_eq!(
        rejected_count, 20,
        "Exactly 20 subsequent requests should be cleanly shed"
    );
}

#[tokio::test]
async fn test_stress_burst_traffic_ip_spread() {
    let harness = Arc::new(TestHarness::new().await);
    let _ = harness
        .live_only()
        .await
        .expect("live Redis required (db-up)");

    // 50 concurrent requests from 50 distinct IPs should all be allowed without false positives
    let worker_count = 50;
    let mut handles = Vec::with_capacity(worker_count);

    let start = Instant::now();

    for i in 0..worker_count {
        let h = Arc::clone(&harness);
        handles.push(tokio::spawn(async move {
            let unique_ip = format!("203.0.113.{}", i + 1);
            let req = Request::builder()
                .method("GET")
                .uri("/health")
                .header(common::CLIENT_IP_HEADER, unique_ip)
                .body(Body::empty())
                .unwrap();

            let resp = h.send_request(req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for handle in handles {
        handle.await.expect("Burst worker panicked");
    }

    let elapsed = start.elapsed();
    println!(
        "Burst test (50 distinct IPs): completed in {} ms",
        elapsed.as_millis()
    );

    assert!(elapsed.as_millis() < 500, "Burst handling took too long");
}
