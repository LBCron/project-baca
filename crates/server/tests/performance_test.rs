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
    let _ = harness
        .live_only()
        .await
        .expect("live Redis required (db-up)");

    let test_email = format!("perf_user_{}@example.com", uuid::Uuid::new_v4());
    let password = "PerfPassword123!".to_string();

    // Setup active test user
    let hash = harness.state.argon2.hash(password.clone()).await.unwrap();
    let _ = infra::activate_registration(&harness.state.db, &test_email, "PerfUser", &hash)
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
            .header(
                common::CLIENT_IP_HEADER,
                format!("10.99.1.{}", (i % 250) + 1),
            )
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
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up)");

    // Warm up the shared DB pool + JWT/Argon2 code paths once so the
    // measured iterations reflect steady state, not cold-start noise.
    {
        let warm_email = format!("perf_otp_warmup_{}@example.com", uuid::Uuid::new_v4());
        if let Ok((registration_id, warm_otp)) =
            infra::start_registration(&mut redis_conn, &warm_email, "PerfUser", "hash").await
        {
            let warm_req = shared::VerifyOtpRequest {
                email: warm_email,
                registration_id,
                otp: warm_otp,
            };
            let req = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/verify-otp")
                .header(common::CLIENT_IP_HEADER, "10.99.2.250")
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
        let (registration_id, otp) =
            infra::start_registration(&mut redis_conn, &test_email, "PerfUser", "hash")
                .await
                .unwrap();

        let verify_req = shared::VerifyOtpRequest {
            email: test_email,
            registration_id,
            otp,
        };

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify-otp")
            .header(
                common::CLIENT_IP_HEADER,
                format!("10.99.2.{}", (i % 250) + 1),
            )
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

/// In-book search reads the paragraph index: a query with no hit in a book
/// of 8 MB of text answers well inside 50 ms at the median, instead of the
/// half second it took to split and strip every chapter per request.
#[tokio::test]
async fn test_perf_in_book_search_on_an_8_mb_book() {
    use sea_orm::{ConnectionTrait, TransactionTrait};
    let harness = TestHarness::new().await;
    if harness.live_only().await.is_none() {
        panic!("live DB required (db-up)");
    }
    let book_id = harness.track_book(uuid::Uuid::new_v4());
    let db = &harness.state.db;
    // Seeding 40,000 rows outlasts the API's statement timeout; lift it for
    // this transaction only.
    let seed = db.begin().await.unwrap();
    seed.execute_unprepared(&format!(
        "SET LOCAL statement_timeout = 0; \
         INSERT INTO books (id, title, author, language, primary_theme, description, cover_url, \
           epub_storage_path, total_words, estimated_reading_minutes, source_name, license, status, \
           created_at, updated_at) \
         VALUES ('{book_id}', 'Perf Tome', 'Perf', 'en', 'Test', '', '', '', 1, 1, 'Test', 'Private', \
           'published', NOW(), NOW()); \
         INSERT INTO chapters (id, book_id, chapter_number, title, word_count, html_content, created_at) \
         SELECT gen_random_uuid(), '{book_id}', c, 'Chapter ' || c, 1, '', NOW() FROM generate_series(1, 40) c; \
         INSERT INTO chapter_paragraphs (book_id, chapter_number, n, text, folded) \
         SELECT '{book_id}', c, n, t, lower(t) FROM ( \
           SELECT c, n, 'The harbour lay quiet ' || md5((c * 10000 + n)::text) \
             || ' while the ships of the ancestral fleet ' || md5(n::text) \
             || ' waited for a wind that would carry them home before the winter came again.' AS t \
           FROM generate_series(1, 40) c, generate_series(1, 1000) n) s;"
    ))
    .await
    .expect("seed the 8 MB book");
    seed.commit().await.unwrap();
    // Earlier runs leave their deleted rows behind until autovacuum comes;
    // measure the index, not that backlog.
    db.execute_unprepared("VACUUM ANALYZE chapter_paragraphs")
        .await
        .expect("vacuum");
    let bytes: i64 = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT sum(length(text))::int8 AS b FROM chapter_paragraphs WHERE book_id = '{book_id}'"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "b")
        .unwrap();
    assert!(bytes >= 8_000_000, "the book holds {bytes} bytes of text");

    let mut latencies = Vec::new();
    for _ in 0..12 {
        let req = Request::builder()
            .uri(format!("/api/v1/books/{book_id}/search?q=zqxjv%20absent"))
            .body(Body::empty())
            .unwrap();
        let start = Instant::now();
        let resp = harness.send_request(req).await;
        latencies.push(start.elapsed().as_micros());
        assert_eq!(resp.status(), StatusCode::OK);
    }
    latencies.sort_unstable();
    let median = latencies[latencies.len() / 2];
    println!("in-book search, 8 MB book, no hit: median {median} µs");
    harness.cleanup().await;
    assert!(median < 50_000, "median {median} µs exceeds 50 ms");
}
