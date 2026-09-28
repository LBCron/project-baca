//! Quote search, saved quotes, cards, and recap tests.

mod common;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use common::TestHarness;
use infra::entities::{book_chunks, books, chapters, tldr_cache, users};
use sea_orm::entity::prelude::PgVector;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::Value;
use uuid::Uuid;

struct SeededAiContext {
    pub book_id: Uuid,
    pub chapter1_id: Uuid,
    pub chapter2_id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub token: String,
    pub other_book_id: Uuid,
    pub other_chapter_id: Uuid,
}

/// Deterministic 768-dim one-hot vector. Index 0 reproduces the
/// `MockEmbeddingProvider` query vector exactly (cosine similarity 1.0);
/// any other index is orthogonal to it (similarity 0.0).
fn one_hot(hot_index: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; 768];
    v[hot_index] = 1.0;
    v
}

/// Second reader for ownership checks (a different user must not see the
/// first user's private quotes).
async fn seed_reader(harness: &TestHarness, email: &str) -> SeededReader {
    let now = Utc::now();
    let user_id = Uuid::new_v4();
    users::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        password_hash: Set(Some("test_password_hash".to_string())),
        display_name: Set("Intruding Reader".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("Intruder seed must succeed");
    let token = infra::generate_access_token(
        user_id,
        email,
        "reader",
        harness.state.config.jwt_secret(),
        1440,
    )
    .expect("Intruder token must generate");
    SeededReader { token }
}

struct SeededReader {
    pub token: String,
}

async fn seed_test_context(harness: &TestHarness) -> Result<SeededAiContext, String> {
    let now = Utc::now();

    // Create a test user and obtain JWT token
    let user_id = Uuid::new_v4();
    let email = format!("ai_connoisseur_{}@baca.local", Uuid::new_v4());
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(Some("test_password_hash".to_string())),
        display_name: Set("Literary AI Explorer".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    user.insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed user: {e}"))?;

    let token = infra::generate_access_token(
        user_id,
        &email,
        "reader",
        harness.state.config.jwt_secret(),
        1440,
    )
    .map_err(|e| format!("Token generation failed: {e}"))?;

    // Create a test book
    let book_id = Uuid::new_v4();
    let book = books::ActiveModel {
        id: Set(book_id),
        title: Set("The Shadow over Innsmouth".to_string()),
        author: Set("H. P. Lovecraft".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Horror".to_string()),
        sub_theme: Set(Some("Weird Fiction".to_string())),
        description: Set("A tale of ancient mysteries and foggy coastal towns.".to_string()),
        cover_url: Set("https://example.com/cover.jpg".to_string()),
        epub_storage_path: Set("/storage/books/test/book.epub".to_string()),
        total_words: Set(42000),
        estimated_reading_minutes: Set(180),
        source_name: Set("Standard Ebooks".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1936)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    book.insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed book: {e}"))?;

    // Create chapter 1
    let chapter1_id = Uuid::new_v4();
    let chapter1 = chapters::ActiveModel {
        id: Set(chapter1_id),
        book_id: Set(book_id),
        chapter_number: Set(1),
        title: Set("Chapter I: The Old Curiosity".to_string()),
        word_count: Set(4200),
        html_content: Set(
            "<p>The oldest and strongest emotion of mankind is fear.</p>".to_string(),
        ),
        created_at: Set(now.into()),
    };

    chapter1
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed chapter 1: {e}"))?;

    // Create chapter 2
    let chapter2_id = Uuid::new_v4();
    let chapter2 = chapters::ActiveModel {
        id: Set(chapter2_id),
        book_id: Set(book_id),
        chapter_number: Set(2),
        title: Set("Chapter II: The Decaying Seaport".to_string()),
        word_count: Set(3800),
        html_content: Set(
            "<p>The crumbling roofs of Innsmouth hung over the dark bay.</p>".to_string(),
        ),
        created_at: Set(now.into()),
    };

    chapter2
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed chapter 2: {e}"))?;

    // Chunks with known vectors: ch1 matches the mock query (sim 1.0),
    // ch2 is orthogonal (sim 0.0).
    let chunk_a = book_chunks::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(book_id),
        chapter_id: Set(chapter1_id),
        chunk_index: Set(0),
        chunk_text: Set(
            "The oldest and strongest emotion of mankind is fear, and the oldest and strongest kind of fear is fear of the unknown."
                .to_string(),
        ),
        embedding: Set(PgVector::from(one_hot(0))),
        created_at: Set(now.into()),
    };

    chunk_a
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed chunk A: {e}"))?;

    let chunk_b = book_chunks::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(book_id),
        chapter_id: Set(chapter2_id),
        chunk_index: Set(0),
        chunk_text: Set(
            "The crumbling roofs of Innsmouth hung over the dark bay as fog swallowed the harbor lights."
                .to_string(),
        ),
        embedding: Set(PgVector::from(one_hot(1))),
        created_at: Set(now.into()),
    };

    chunk_b
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed chunk B: {e}"))?;

    // Second book proving per-book scope isolation.
    let other_book_id = Uuid::new_v4();
    let other_book = books::ActiveModel {
        id: Set(other_book_id),
        title: Set("The Call of Cthulhu".to_string()),
        author: Set("H. P. Lovecraft".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Horror".to_string()),
        sub_theme: Set(Some("Weird Fiction".to_string())),
        description: Set("A cult stirs beneath the Pacific ocean.".to_string()),
        cover_url: Set("https://example.com/cthulhu.jpg".to_string()),
        epub_storage_path: Set("/storage/books/test/cthulhu.epub".to_string()),
        total_words: Set(12000),
        estimated_reading_minutes: Set(50),
        source_name: Set("Standard Ebooks".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1928)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    other_book
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed other book: {e}"))?;

    let other_chapter_id = Uuid::new_v4();
    let other_chapter = chapters::ActiveModel {
        id: Set(other_chapter_id),
        book_id: Set(other_book_id),
        chapter_number: Set(1),
        title: Set("Chapter I: The Horror in Clay".to_string()),
        word_count: Set(3000),
        html_content: Set("<p>Ph'nglui mglw'nafh Cthulhu R'lyeh wgah'nagl fhtagn.</p>".to_string()),
        created_at: Set(now.into()),
    };

    other_chapter
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed other chapter: {e}"))?;

    let other_chunk = book_chunks::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(other_book_id),
        chapter_id: Set(other_chapter_id),
        chunk_index: Set(0),
        chunk_text: Set("Ph'nglui mglw'nafh Cthulhu R'lyeh wgah'nagl fhtagn.".to_string()),
        embedding: Set(PgVector::from(one_hot(0))),
        created_at: Set(now.into()),
    };

    other_chunk
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed other chunk: {e}"))?;

    Ok(SeededAiContext {
        book_id,
        chapter1_id,
        chapter2_id,
        user_id,
        token,
        other_book_id,
        other_chapter_id,
    })
}

#[tokio::test]
async fn test_scoped_quote_search_open_to_guests_and_validation() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Guest requests succeed without a token.
    let guest_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "query": "foggy harbor" }).to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(guest_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    // Honest relevance: the seeded book holds embedded chunks, so guests
    // must get real results back — not an empty array.
    let guest_data = json["data"].as_array().expect("data must be an array");
    assert!(
        !guest_data.is_empty(),
        "guest search must return seeded chunks, got {guest_data:?}"
    );

    // Empty query string must fail validation (min length 3)
    let invalid_empty_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "query": "" }).to_string()))
        .expect("Valid request");

    let resp = harness.send_request(invalid_empty_req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Query shorter than 3 chars must fail validation
    let invalid_short_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "query": "ab" }).to_string()))
        .expect("Valid request");

    let resp = harness.send_request(invalid_short_req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Query with limit > 20 must fail validation
    let invalid_limit_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "query": "foggy harbor", "limit": 50 }).to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(invalid_limit_req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_scoped_quote_search_success_and_ratelimit_headers() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": "hesitation in foggy night",
                "limit": 5
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Rate limit headers must be present when Redis is available
    if let Some(limit_val) = resp.headers().get("x-ratelimit-limit") {
        assert_eq!(limit_val, "10");
        assert!(resp.headers().get("x-ratelimit-remaining").is_some());
        assert!(resp.headers().get("x-ratelimit-reset").is_some());
    }

    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    let data = json["data"].as_array().expect("data must be an array");
    // Honest relevance: the seeded book holds exactly 2 embedded chunks.
    assert_eq!(
        data.len(),
        2,
        "seeded book must return its 2 chunks, got {data:?}"
    );
    // The mock query vector matches the chapter-1 chunk exactly (similarity
    // ~1.0); the orthogonal chapter-2 chunk ranks last.
    let top = &data[0];
    assert_eq!(top["chapter_number"], 1);
    let top_sim = top["similarity_score"]
        .as_f64()
        .expect("similarity_score must be a number");
    assert!(
        (top_sim - 1.0).abs() < 1e-5,
        "exact-match chunk must score ~1.0, got {top_sim}"
    );
    assert!(top_sim > 0.65);
    let second_sim = data[1]["similarity_score"]
        .as_f64()
        .expect("similarity_score must be a number");
    assert!(
        second_sim < top_sim,
        "orthogonal chunk must rank below the exact match"
    );
    // Scope isolation: the other book's chunk must not leak in.
    for item in data {
        let content = item["content"].as_str().expect("content must be a string");
        assert!(
            !content.contains("Cthulhu"),
            "search leaked another book's chunk: {content}"
        );
    }
}

#[tokio::test]
async fn test_scoped_quote_search_isolated_per_book() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Search scoped to the second book returns only its own chunk.
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/books/{}/quotes/search",
            seeded.other_book_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "query": "ancient cult beneath the ocean",
                "limit": 5
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    let data = json["data"].as_array().expect("data must be an array");
    assert_eq!(
        data.len(),
        1,
        "other book holds exactly 1 chunk, got {data:?}"
    );
    assert!(data[0]["content"]
        .as_str()
        .expect("content must be a string")
        .contains("Cthulhu"));
}

#[tokio::test]
async fn test_ai_rate_limiter_burst_enforcement() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Make 10 requests that should all be accepted
    for _ in 1..=10 {
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::json!({ "query": "ancient mysteries", "limit": 2 }).to_string(),
            ))
            .expect("Valid request");

        let resp = harness.send_request(req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // 11th request from same user
    let overflow_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "query": "ancient mysteries", "limit": 2 }).to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(overflow_req).await;

    // If Redis is active in test environment, 11th request must return 429
    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
        assert_eq!(resp.headers().get("x-ratelimit-remaining").unwrap(), "0");
        assert!(resp.headers().get(header::RETRY_AFTER).is_some());

        let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"]["code"], "AI_RATE_LIMITED");
    }
}

#[tokio::test]
async fn test_scoped_quote_search_empty_book_returns_empty_array() {
    let harness = TestHarness::new().await;
    let now = Utc::now();
    let empty_book_id = Uuid::new_v4();
    books::ActiveModel {
        id: Set(empty_book_id),
        title: Set("Empty Shelves".to_string()),
        author: Set("Nobody".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Test".to_string()),
        sub_theme: Set(None),
        description: Set(String::new()),
        cover_url: Set(String::new()),
        epub_storage_path: Set(String::new()),
        total_words: Set(0),
        estimated_reading_minutes: Set(0),
        source_name: Set("Test".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(None),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("Empty book seed must succeed");

    // A published book without chunks yields 200 with zero results.
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{empty_book_id}/quotes/search"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "query": "anything at all", "limit": 5 }).to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["data"].as_array().expect("data array").len(), 0);
}

#[tokio::test]
async fn test_guest_ai_burst_enforcement() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Unique IP per run isolates this bucket from other tests and re-runs.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock must advance")
        .as_nanos();
    let guest_ip = format!("198.51.100.{}", (nanos % 200 + 1) as u8);

    let mut ok_count = 0;
    for _ in 0..25 {
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/books/{}/quotes/search", seeded.book_id))
            .header("cf-connecting-ip", &guest_ip)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::json!({ "query": "foggy harbor", "limit": 2 }).to_string(),
            ))
            .expect("Valid request");
        let resp = harness.send_request(req).await;
        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            let has_retry = resp.headers().get(header::RETRY_AFTER).is_some();
            let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["error"]["code"], "AI_RATE_LIMITED");
            assert!(has_retry);
            break;
        }
        assert_eq!(resp.status(), StatusCode::OK);
        ok_count += 1;
    }
    assert!(ok_count >= 1, "guest burst must allow initial requests");
    assert!(ok_count <= 10, "guest burst must shed past 10 req/min");
}

#[tokio::test]
async fn test_quote_search_unknown_book_returns_404() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Unknown books must 404, never return an empty 200.
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/books/{}/quotes/search", Uuid::new_v4()))
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "query": "anything at all", "limit": 5 }).to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_save_quote_rejects_unknown_book_and_mismatched_chapter() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Unauthenticated saves are rejected before any persistence.
    let anon_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "book_id": seeded.book_id,
                "chapter_id": seeded.chapter1_id,
                "quote_text": "A quote with no credentials attached."
            })
            .to_string(),
        ))
        .expect("Valid request");
    let resp = harness.send_request(anon_req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Unknown book_id must yield 404, not a 500 FK violation.
    let bogus_book_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "book_id": Uuid::new_v4(),
                "chapter_id": seeded.chapter1_id,
                "quote_text": "A quote attached to a book that does not exist."
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(bogus_book_req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Unknown chapter_id must yield 404.
    let bogus_chapter_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "book_id": seeded.book_id,
                "chapter_id": Uuid::new_v4(),
                "quote_text": "A quote attached to a chapter that does not exist."
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(bogus_chapter_req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cross-book mismatch (real chapter of another book) must yield 400.
    let mismatch_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "book_id": seeded.other_book_id,
                "chapter_id": seeded.chapter1_id,
                "quote_text": "A quote pairing a book with another book's chapter."
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(mismatch_req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "VALIDATION_FAILED");

    // Control: correct pair still saves (uses the other book's own chapter).
    let ok_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "book_id": seeded.other_book_id,
                "chapter_id": seeded.other_chapter_id,
                "quote_text": "Ph'nglui mglw'nafh Cthulhu R'lyeh wgah'nagl fhtagn."
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(ok_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_saved_quotes_and_vintage_card_export() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Authenticated save persists quote and creates image_card_url
    let auth_save_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "book_id": seeded.book_id,
                "chapter_id": seeded.chapter1_id,
                "quote_text": "The oldest and strongest emotion of mankind is fear, and the oldest and strongest kind of fear is fear of the unknown."
            })
            .to_string(),
        ))
        .expect("Valid request");

    let resp = harness.send_request(auth_save_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["saved"], true);
    let quote_id = json["data"]["id"].as_str().expect("Quote ID string");
    let card_url = json["data"]["image_card_url"].as_str().expect("Card URL");
    assert!(card_url.contains("/card"));

    // Vintage SVG quote card export endpoint returns valid SVG
    let card_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/quotes/{quote_id}/card"))
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(card_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/svg+xml; charset=utf-8"
    );
    let svg_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let svg_str = String::from_utf8(svg_bytes.to_vec()).unwrap();
    assert!(svg_str.contains("<svg"));
    assert!(svg_str.contains("PROJECT BACA"));
    assert!(svg_str.contains("The oldest and strongest emotion"));

    // Another user cannot render this quote card: same id, wrong owner.
    let other = seed_reader(&harness, &format!("intruder_{}@baca.local", Uuid::new_v4())).await;
    let intruder_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/quotes/{quote_id}/card"))
        .header(header::AUTHORIZATION, format!("Bearer {}", other.token))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(intruder_req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Unauthenticated card access is rejected.
    let anon_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/quotes/{quote_id}/card"))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(anon_req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Authenticated list retrieves saved quotes with image_card_url
    let list_req = Request::builder()
        .method("GET")
        .uri("/api/v1/quotes")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(list_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    let quotes = json["data"].as_array().expect("Array of saved quotes");
    assert!(!quotes.is_empty());
    assert!(quotes[0]["image_card_url"].is_string());
}

#[tokio::test]
async fn test_atomic_cards_scoped_to_path_book() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Seed cards for chapter 1 of the main book.
    tldr_cache::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(seeded.book_id),
        chapter_id: Set(Some(seeded.chapter1_id)),
        recap_type: Set("chapter_atomic_cards".to_string()),
        content_json: Set(serde_json::json!({"key_concepts": ["fear"]})),
        model_version: Set("test".to_string()),
        created_at: Set(Utc::now().into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("tldr seed must succeed");

    // Same chapter UUID under the other book must 404 (scoped lookup).
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/books/{}/chapters/{}/atomic-cards",
            seeded.other_book_id, seeded.chapter1_id
        ))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_chapter1_recap_unavailable_by_uuid() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Even with a recap row present, chapter 1 stays recap-free by UUID too.
    tldr_cache::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(seeded.book_id),
        chapter_id: Set(Some(seeded.chapter1_id)),
        recap_type: Set("chapter_recap".to_string()),
        content_json: Set(serde_json::json!({"summary": "stale"})),
        model_version: Set("test".to_string()),
        created_at: Set(Utc::now().into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("tldr seed must succeed");

    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/books/{}/chapters/{}/recap",
            seeded.book_id, seeded.chapter1_id
        ))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_atomic_insight_cards_cache_hit_and_miss() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Cache miss returns 404 (by chapter number or UUID)
    let miss_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/books/{}/chapters/1/atomic-cards",
            seeded.book_id
        ))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(miss_req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Seed atomic cards into tldr_cache for chapter 1
    let now = Utc::now();
    let cards_content = serde_json::json!({
        "key_concepts": ["Ancient cosmic terror", "Innsmouth degeneration"],
        "notable_quotes": ["The oldest and strongest emotion of mankind is fear."],
        "historical_context": "Written in late 1931, exploring isolated New England seaports."
    });

    let cache_entry = tldr_cache::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(seeded.book_id),
        chapter_id: Set(Some(seeded.chapter1_id)),
        recap_type: Set("chapter_atomic_cards".to_string()),
        content_json: Set(cards_content),
        model_version: Set("gemini-embedding-2".to_string()),
        created_at: Set(now.into()),
    };

    cache_entry
        .insert(&harness.state.db)
        .await
        .expect("Insert tldr_cache entry");

    // Cache hit returns 200 via chapter_number (/chapters/1/atomic-cards)
    let hit_num_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/books/{}/chapters/1/atomic-cards",
            seeded.book_id
        ))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(hit_num_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["book_id"], seeded.book_id.to_string());
    assert_eq!(json["data"]["chapter_id"], seeded.chapter1_id.to_string());
    assert!(json["data"]["cards"]["key_concepts"].is_array());

    // Cache hit also returns 200 via UUID (/chapters/{uuid}/atomic-cards)
    let hit_uuid_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/books/{}/chapters/{}/atomic-cards",
            seeded.book_id, seeded.chapter1_id
        ))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(hit_uuid_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_chapter_recap_by_number_and_uuid() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_context(&harness)
        .await
        .expect("Seeding must succeed");

    // Chapter 1 never has a recap (nothing prior to summarize).
    let chap1_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/books/{}/chapters/1/recap", seeded.book_id))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(chap1_req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Chapter 2 without cache entry returns 404
    let miss_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/books/{}/chapters/2/recap", seeded.book_id))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(miss_req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Seed recap for chapter 2
    let now = Utc::now();
    let recap_content = serde_json::json!({
        "summary": "In previous chapters, the narrator arrived in Newburyport and heard sinister rumors about Innsmouth.",
        "key_characters": ["Robert Olmstead", "Zadok Allen"],
        "spoiler_free_note": "This recap covers only confirmed events through chapter 1."
    });

    let cache_entry = tldr_cache::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(seeded.book_id),
        chapter_id: Set(Some(seeded.chapter2_id)),
        recap_type: Set("chapter_recap".to_string()),
        content_json: Set(recap_content),
        model_version: Set("gemini-embedding-2".to_string()),
        created_at: Set(now.into()),
    };

    cache_entry
        .insert(&harness.state.db)
        .await
        .expect("Insert tldr_cache entry");

    // Cache hit returns 200 when querying by chapter number: /chapters/2/recap
    let hit_num_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/books/{}/chapters/2/recap", seeded.book_id))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(hit_num_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["book_id"], seeded.book_id.to_string());
    assert_eq!(json["data"]["chapter_id"], seeded.chapter2_id.to_string());
    assert!(json["data"]["recap"]["summary"].is_string());

    // Cache hit returns 200 when querying by chapter UUID
    let hit_uuid_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/books/{}/chapters/{}/recap",
            seeded.book_id, seeded.chapter2_id
        ))
        .body(Body::empty())
        .expect("Valid request");

    let resp = harness.send_request(hit_uuid_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// Split card ownership (owner 200 / intruder 404 / anon 401) +
// insights unknown-book 404. Needs live DB; panics loudly without it.

/// Quote card: owner renders 200 SVG, intruder gets 404, anonymous gets 401.
#[tokio::test]
async fn test_quote_card_ownership_split() {
    let harness = TestHarness::new().await;
    let seeded = match seed_test_context(&harness).await {
        Ok(s) => s,
        Err(e) => panic!("live DB required (db-up), no silent pass: {e}"),
    };
    // Save one quote as the owner.
    let save_req = Request::builder()
        .method("POST")
        .uri("/api/v1/quotes/save")
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "book_id": seeded.book_id,
                "chapter_id": seeded.chapter1_id,
                "quote_text": "Ownership split probe quote."
            }))
            .unwrap(),
        ))
        .expect("Valid request");
    let resp = harness.send_request(save_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let raw = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&raw).unwrap();
    let quote_id = json["data"]["id"].as_str().expect("quote id").to_string();

    // Owner: 200 SVG.
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/quotes/{quote_id}/card"))
        .header(header::AUTHORIZATION, format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .contains("svg"));

    // Intruder: 404 (existence not leaked).
    let other = seed_reader(&harness, &format!("split_{}@baca.local", Uuid::new_v4())).await;
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/quotes/{quote_id}/card"))
        .header(header::AUTHORIZATION, format!("Bearer {}", other.token))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Anonymous: 401.
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/quotes/{quote_id}/card"))
        .body(Body::empty())
        .expect("Valid request");
    let resp = harness.send_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Insights endpoints on an unknown book: 404 (never 404-cache-miss
/// confusion, never 500).
#[tokio::test]
async fn test_insights_unknown_book_returns_404() {
    let harness = TestHarness::new().await;
    if matches!(harness.state.db, sea_orm::DatabaseConnection::Disconnected) {
        panic!("live DB required (db-up), no silent pass");
    }
    let ghost = Uuid::new_v4();
    for uri in [
        format!("/api/v1/books/{ghost}/chapters/2/atomic-cards"),
        format!("/api/v1/books/{ghost}/chapters/2/recap"),
    ] {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .expect("Valid request");
        let resp = harness.send_request(req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
