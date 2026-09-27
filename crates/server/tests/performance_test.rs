//! p95 latency and concurrency benchmarks.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestHarness;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_perf_latency_sla_p95() {
    let harness = TestHarness::new().await;
    let iterations = 200;
    let mut latencies_micros = Vec::with_capacity(iterations);

    // Warm-up requests
    for _ in 0..10 {
        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let _ = harness.send_request(req).await;
    }

    // Benchmark loop
    for _ in 0..iterations {
        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let start = Instant::now();
        let resp = harness.send_request(req).await;
        let duration = start.elapsed();

        assert_eq!(resp.status(), StatusCode::OK);
        latencies_micros.push(duration.as_micros());
    }

    latencies_micros.sort_unstable();

    let p50_micros = latencies_micros[(iterations as f64 * 0.50) as usize];
    let p95_micros = latencies_micros[(iterations as f64 * 0.95) as usize];
    let p99_micros = latencies_micros[(iterations as f64 * 0.99) as usize];

    println!(
        "Benchmark (/health): p50 = {} µs, p95 = {} µs, p99 = {} µs",
        p50_micros, p95_micros, p99_micros
    );

    let sla_p95_limit_micros = 50_000;
    assert!(
        p95_micros < sla_p95_limit_micros,
        "p95 latency ({} µs) exceeds SLA limit ({} µs)",
        p95_micros,
        sla_p95_limit_micros
    );
}

#[tokio::test]
async fn test_perf_concurrent_load_50_workers() {
    let harness = Arc::new(TestHarness::new().await);
    let worker_count = 50;
    let mut join_handles = Vec::with_capacity(worker_count);

    let start_total = Instant::now();

    for i in 0..worker_count {
        let h = Arc::clone(&harness);
        join_handles.push(tokio::spawn(async move {
            let req = Request::builder()
                .method("GET")
                .uri("/health")
                .header("x-request-id", format!("concurrent_worker_{i}"))
                .body(Body::empty())
                .unwrap();

            let resp = h.send_request(req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }));
    }

    for handle in join_handles {
        handle.await.expect("Worker task panicked");
    }

    let elapsed = start_total.elapsed();
    println!(
        "Concurrent load: 50 workers completed in {} ms ({:.2} ms per worker)",
        elapsed.as_millis(),
        elapsed.as_secs_f64() * 1000.0 / (worker_count as f64)
    );

    assert!(
        elapsed.as_millis() < 1000,
        "Concurrency execution exceeded 1s: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_perf_auth_login_latency() {
    let harness = TestHarness::new().await;
    if harness.state.get_redis_conn().await.is_err() {
        println!("Redis not reachable, skipping auth login latency test");
        return;
    }

    let test_email = format!("perf_user_{}@example.com", uuid::Uuid::new_v4());
    let password = "PerfPassword123!".to_string();

    // Setup active test user
    let hash = infra::hash_password_async(password.clone()).await.unwrap();
    let _ = infra::create_inactive_user(&harness.state.db, &test_email, "PerfUser", &hash)
        .await
        .unwrap();
    let _ = infra::activate_user_by_email(&harness.state.db, &test_email)
        .await
        .unwrap();

    let iterations = 20;
    let mut latencies_ms = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let login_req = shared::LoginRequest {
            email: test_email.clone(),
            password: password.clone(),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("cf-connecting-ip", format!("10.99.1.{}", (i % 250) + 1))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&login_req).unwrap()))
            .unwrap();

        let start = Instant::now();
        let resp = harness.send_request(req).await;
        let duration = start.elapsed();

        assert_eq!(resp.status(), StatusCode::OK);
        latencies_ms.push(duration.as_millis());
    }

    latencies_ms.sort_unstable();
    let p50_ms = latencies_ms[(iterations as f64 * 0.50) as usize];
    let p95_ms = latencies_ms[(iterations as f64 * 0.95) as usize];

    println!(
        "Benchmark (/api/v1/auth/login): p50 = {} ms, p95 = {} ms",
        p50_ms, p95_ms
    );

    // Cryptographic Argon2id login SLA target: p95 < 150ms
    assert!(
        p95_ms < 150,
        "Login p95 latency ({} ms) exceeded target (150 ms)",
        p95_ms
    );
}

#[tokio::test]
async fn test_perf_auth_otp_verify_latency() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.state.get_redis_conn().await {
        Ok(c) => c,
        Err(_) => {
            println!("Redis not reachable, skipping OTP verify latency test");
            return;
        }
    };

    // Warm up the shared DB pool + JWT/Argon2 code paths once so the
    // measured iterations reflect steady state, not cold-start noise.
    {
        let warm_email = format!("perf_otp_warmup_{}@example.com", uuid::Uuid::new_v4());
        let _ =
            infra::create_inactive_user(&harness.state.db, &warm_email, "PerfUser", "hash").await;
        if let Ok(warm_otp) = infra::generate_and_store_otp(&mut redis_conn, &warm_email).await {
            let warm_req = shared::VerifyOtpRequest {
                email: warm_email,
                otp: warm_otp,
            };
            let req = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/verify-otp")
                .header("cf-connecting-ip", "10.99.2.250")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&warm_req).unwrap()))
                .unwrap();
            let _ = harness.send_request(req).await;
        }
    }

    let iterations = 20;
    let mut latencies_ms = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let test_email = format!("perf_otp_{}_{}@example.com", i, uuid::Uuid::new_v4());
        let _ = infra::create_inactive_user(&harness.state.db, &test_email, "PerfUser", "hash")
            .await
            .unwrap();
        let otp = infra::generate_and_store_otp(&mut redis_conn, &test_email)
            .await
            .unwrap();

        let verify_req = shared::VerifyOtpRequest {
            email: test_email,
            otp,
        };

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify-otp")
            .header("cf-connecting-ip", format!("10.99.2.{}", (i % 250) + 1))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&verify_req).unwrap()))
            .unwrap();

        let start = Instant::now();
        let resp = harness.send_request(req).await;
        let duration = start.elapsed();

        assert_eq!(resp.status(), StatusCode::OK);
        latencies_ms.push(duration.as_millis());
    }

    latencies_ms.sort_unstable();
    let p50_ms = latencies_ms[(iterations as f64 * 0.50) as usize];
    let p95_ms = latencies_ms[(iterations as f64 * 0.95) as usize];

    println!(
        "Benchmark (/api/v1/auth/verify-otp): p50 = {} ms, p95 = {} ms",
        p50_ms, p95_ms
    );

    // Fast OTP verify SLA target: p95 < 100ms (steady state; Argon2id
    // activation hashing plus live Postgres/Redis round trips).
    assert!(
        p95_ms < 100,
        "OTP verify p95 latency ({} ms) exceeded target (100 ms)",
        p95_ms
    );
}
