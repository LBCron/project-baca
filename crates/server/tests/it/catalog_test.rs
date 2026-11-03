//! Catalog, search, reader, and gamification tests.

use crate::common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use common::TestHarness;
use infra::entities::{book_tags, books, chapters, tags, users};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

struct SeededCatalog {
    pub book1_id: Uuid,
    #[allow(dead_code)]
    pub book2_id: Uuid,
    #[allow(dead_code)]
    pub book3_id: Uuid,
    #[allow(dead_code)]
    pub draft_id: Uuid,
    pub chap1_id: Uuid,
    #[allow(dead_code)]
    pub chap2_id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub token: String,
}

async fn seed_test_catalog(harness: &TestHarness) -> Result<SeededCatalog, String> {
    let now = Utc::now();

    // Create a test user and obtain JWT token
    let user_id = Uuid::new_v4();
    let email = format!("reader_{}@baca.local", Uuid::new_v4());
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(Some("test_password_hash".to_string())),
        display_name: Set("Literary Connoisseur".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    user.insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed user: {e}"))?;
    harness.track_user(user_id);

    let token = infra::generate_access_token(
        user_id,
        &email,
        "reader",
        harness.state.config.jwt_secret(),
        1440,
    )
    .map_err(|e| format!("Token generation failed: {e}"))?;

    // Seed Tags
    let tag_id = Uuid::new_v4();
    let tag_slug = format!("klasik-{}", Uuid::new_v4());
    let tag = tags::ActiveModel {
        id: Set(tag_id),
        name: Set(format!("Klasik Nusantara {}", tag_id)),
        slug: Set(tag_slug.clone()),
        created_at: Set(now.into()),
    };
    let _ = tag.insert(&harness.state.db).await;
    harness.track_tag(tag_id);

    // Seed Published Books
    let book1_id = Uuid::new_v4();
    let book1 = books::ActiveModel {
        id: Set(book1_id),
        title: Set("Sitti Nurbaya Kasih Tak Sampai".to_string()),
        author: Set("Marah Rusli".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Tragedi".to_string()),
        sub_theme: Set(Some("Romansa Klasik".to_string())),
        description: Set(
            "Kisah cinta sejati antara Samsulbahri dan Sitti Nurbaya di Padang tempo dulu."
                .to_string(),
        ),
        cover_url: Set("https://assets.baca.local/covers/sitti.jpg".to_string()),
        epub_storage_path: Set("epubs/sitti_nurbaya.epub".to_string()),
        total_words: Set(85000),
        estimated_reading_minutes: Set(340),
        source_name: Set("Balai Pustaka".to_string()),
        source_url: Set(Some("https://balaipustaka.co.id".to_string())),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1922)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    book1
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed book 1: {e}"))?;
    harness.track_book(book1_id);

    // Connect tag to book 1
    let book_tag = book_tags::ActiveModel {
        book_id: Set(book1_id),
        tag_id: Set(tag_id),
    };
    let _ = book_tag.insert(&harness.state.db).await;

    let book2_id = Uuid::new_v4();
    let book2 = books::ActiveModel {
        id: Set(book2_id),
        title: Set("Salah Asuhan".to_string()),
        author: Set("Abdul Muis".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Drama".to_string()),
        sub_theme: Set(Some("Konflik Budaya".to_string())),
        description: Set("Pertentangan adat dan modernitas pada zaman kolonial.".to_string()),
        cover_url: Set("https://assets.baca.local/covers/salah.jpg".to_string()),
        epub_storage_path: Set("epubs/salah_asuhan.epub".to_string()),
        total_words: Set(62000),
        estimated_reading_minutes: Set(250),
        source_name: Set("Balai Pustaka".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1928)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    book2
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed book 2: {e}"))?;
    harness.track_book(book2_id);

    let book3_id = Uuid::new_v4();
    let book3 = books::ActiveModel {
        id: Set(book3_id),
        title: Set("Bumi Manusia".to_string()),
        author: Set("Pramoedya Ananta Toer".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Sejarah Perjuangan".to_string()),
        sub_theme: Set(Some("Humanisme".to_string())),
        description: Set(
            "Perjalanan pemuda Minke dan pergulatan martabat pribumi di Wonokromo.".to_string(),
        ),
        cover_url: Set("https://assets.baca.local/covers/bumi.jpg".to_string()),
        epub_storage_path: Set("epubs/bumi_manusia.epub".to_string()),
        total_words: Set(120000),
        estimated_reading_minutes: Set(480),
        source_name: Set("Hasta Mitra".to_string()),
        source_url: Set(None),
        license: Set("Creative Commons BY-NC".to_string()),
        publication_year: Set(Some(1980)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    book3
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed book 3: {e}"))?;
    harness.track_book(book3_id);

    // Seed Draft Book (Must NOT appear in catalog)
    let draft_id = Uuid::new_v4();
    let draft_book = books::ActiveModel {
        id: Set(draft_id),
        title: Set("Unpublished Draft Manuscript".to_string()),
        author: Set("Secret Author".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Misteri".to_string()),
        sub_theme: Set(None),
        description: Set("Draft that should be invisible to regular readers.".to_string()),
        cover_url: Set("https://assets.baca.local/covers/draft.jpg".to_string()),
        epub_storage_path: Set("epubs/draft.epub".to_string()),
        total_words: Set(1000),
        estimated_reading_minutes: Set(5),
        source_name: Set("Draft".to_string()),
        source_url: Set(None),
        license: Set("Private".to_string()),
        publication_year: Set(Some(2026)),
        status: Set("draft".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    draft_book
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed draft book: {e}"))?;
    harness.track_book(draft_id);

    // Seed Chapters for Book 1
    let chap1_id = Uuid::new_v4();
    let chap1 = chapters::ActiveModel {
        id: Set(chap1_id),
        book_id: Set(book1_id),
        chapter_number: Set(1),
        title: Set("Di Bukit Padang".to_string()),
        word_count: Set(3500),
        html_content: Set(
            "<p>Matahari senja perlahan turun di balik perbukitan Padang...</p>".to_string(),
        ),
        created_at: Set(now.into()),
    };
    chap1
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed chapter 1: {e}"))?;

    let chap2_id = Uuid::new_v4();
    let chap2 = chapters::ActiveModel {
        id: Set(chap2_id),
        book_id: Set(book1_id),
        chapter_number: Set(2),
        title: Set("Pertemuan Rahasia".to_string()),
        word_count: Set(4200),
        html_content: Set(
            "<p>Surapati berdiri dalam keheningan malam yang sunyi...</p>".to_string(),
        ),
        created_at: Set(now.into()),
    };
    chap2
        .insert(&harness.state.db)
        .await
        .map_err(|e| format!("Failed to seed chapter 2: {e}"))?;

    Ok(SeededCatalog {
        book1_id,
        book2_id,
        book3_id,
        draft_id,
        chap1_id,
        chap2_id,
        user_id,
        token,
    })
}

#[tokio::test]
async fn test_catalog_listing_and_filtering() {
    let harness = TestHarness::new().await;
    let _seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");

    // List all books
    let req = Request::builder()
        .uri("/api/v1/books")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp, json) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(json["success"].as_bool().unwrap_or(false));

    let books_array = json["data"].as_array().expect("Expected data array");
    assert!(!books_array.is_empty());

    // Verify draft book is NOT listed
    for b in books_array {
        let title = b["title"].as_str().unwrap_or("");
        assert_ne!(
            title, "Unpublished Draft Manuscript",
            "Draft book must not appear in public catalog"
        );
    }

    // Filter by theme
    let req_theme = Request::builder()
        .uri("/api/v1/books?theme=Tragedi")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_theme, json_theme) = harness.send_json_request(req_theme).await;
    assert_eq!(resp_theme.status(), StatusCode::OK);
    let filtered = json_theme["data"].as_array().unwrap();
    assert!(!filtered.is_empty());
    for b in filtered {
        assert_eq!(b["primary_theme"].as_str().unwrap_or(""), "Tragedi");
    }

    // The unversioned `/api/*` alias is gone: one documented surface only.
    let req_alias = Request::builder()
        .uri("/api/books?language=id")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_alias, json_alias) = harness.send_json_request(req_alias).await;
    assert_eq!(resp_alias.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_alias["error"]["code"], "NOT_FOUND");
    harness.cleanup().await;
}

#[tokio::test]
async fn test_catalog_typo_tolerant_fts_search() {
    let harness = TestHarness::new().await;
    let _seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");

    // Search with typo: "Siti" instead of "Sitti"
    let req = Request::builder()
        .uri("/api/v1/books/search?q=Siti")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp, json) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let results = json["data"].as_array().expect("Expected results array");
    assert!(
        !results.is_empty(),
        "Trigram search should match 'Siti' to 'Sitti Nurbaya'"
    );
    assert!(results[0]["title"].as_str().unwrap_or("").contains("Sitti"));

    // Search author name: "Pramoedya"
    let req_author = Request::builder()
        .uri("/api/v1/books/search?q=Pramoedya")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_author, json_author) = harness.send_json_request(req_author).await;
    assert_eq!(resp_author.status(), StatusCode::OK);
    let author_results = json_author["data"].as_array().unwrap();
    assert!(!author_results.is_empty());
    assert!(author_results[0]["title"]
        .as_str()
        .unwrap_or("")
        .contains("Bumi Manusia"));

    // Search query too short (min 2 characters)
    let req_short = Request::builder()
        .uri("/api/v1/books/search?q=a")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_short, _) = harness.send_json_request(req_short).await;
    assert_eq!(resp_short.status(), StatusCode::BAD_REQUEST);
    harness.cleanup().await;
}

#[tokio::test]
async fn test_book_overview_chapter_and_offline_bundle() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");

    // Get book detail
    let req_book = Request::builder()
        .uri(format!("/api/v1/books/{}", seeded.book1_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_book, json_book) = harness.send_json_request(req_book).await;
    assert_eq!(resp_book.status(), StatusCode::OK);
    assert_eq!(
        json_book["data"]["title"].as_str().unwrap_or(""),
        "Sitti Nurbaya Kasih Tak Sampai"
    );
    assert_eq!(json_book["data"]["chapters"].as_array().unwrap().len(), 2);

    // Get chapter content
    let req_chap = Request::builder()
        .uri(format!("/api/v1/books/{}/chapters/1", seeded.book1_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_chap, json_chap) = harness.send_json_request(req_chap).await;
    assert_eq!(resp_chap.status(), StatusCode::OK);
    let html = json_chap["data"]["html_content"].as_str().unwrap_or("");
    assert!(html.contains("Matahari senja perlahan turun"));

    // Get offline bundle
    let req_bundle = Request::builder()
        .uri(format!("/api/v1/books/{}/offline-bundle", seeded.book1_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_bundle, json_bundle) = harness.send_json_request(req_bundle).await;
    assert_eq!(resp_bundle.status(), StatusCode::OK);
    assert!(json_bundle["data"]["book"].is_object());
    assert_eq!(json_bundle["data"]["chapters"].as_array().unwrap().len(), 2);
    harness.cleanup().await;
}

#[tokio::test]
async fn test_reading_progress_and_active_retrieval() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");

    // Update progress to 45%
    let progress_payload = serde_json::json!({
        "chapter_id": seeded.chap1_id,
        "last_anchor_cfi": "/6/4[chap1]!/4/2/10",
        "completion_percentage": 45.0
    });

    let req_update = Request::builder()
        .uri(format!("/api/v1/progress/{}", seeded.book1_id))
        .method("PUT")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .header("Content-Type", "application/json")
        .body(Body::from(progress_payload.to_string()))
        .unwrap();

    let (resp_update, json_update) = harness.send_json_request(req_update).await;
    assert_eq!(resp_update.status(), StatusCode::OK);
    assert!(json_update["success"].as_bool().unwrap_or(false));

    // Retrieve active progress
    let req_active = Request::builder()
        .uri("/api/v1/progress/active")
        .method("GET")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .unwrap();

    let (resp_active, json_active) = harness.send_json_request(req_active).await;
    assert_eq!(resp_active.status(), StatusCode::OK);
    assert_eq!(
        json_active["data"]["book_id"].as_str().unwrap_or(""),
        seeded.book1_id.to_string()
    );
    assert_eq!(
        json_active["data"]["chapter_id"].as_str().unwrap_or(""),
        seeded.chap1_id.to_string()
    );
    assert_eq!(
        json_active["data"]["last_anchor_cfi"]
            .as_str()
            .unwrap_or(""),
        "/6/4[chap1]!/4/2/10"
    );

    // Mark finished at 100%
    let finished_payload = serde_json::json!({
        "chapter_id": seeded.chap1_id,
        "last_anchor_cfi": "/6/4[chap1]!/4/2/99",
        "completion_percentage": 100.0
    });

    let req_finish = Request::builder()
        .uri(format!("/api/v1/progress/{}", seeded.book1_id))
        .method("PUT")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .header("Content-Type", "application/json")
        .body(Body::from(finished_payload.to_string()))
        .unwrap();

    let (resp_finish, _) = harness.send_json_request(req_finish).await;
    assert_eq!(resp_finish.status(), StatusCode::OK);

    // Verify active progress is now null
    let req_active2 = Request::builder()
        .uri("/api/v1/progress/active")
        .method("GET")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .unwrap();

    let (resp_active2, json_active2) = harness.send_json_request(req_active2).await;
    assert_eq!(resp_active2.status(), StatusCode::OK);
    assert!(json_active2["data"].is_null());
    harness.cleanup().await;
}

#[tokio::test]
async fn test_gamification_heartbeat_and_badges() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");

    // List master badges
    let req_badges = Request::builder()
        .uri("/api/v1/badges")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_badges, json_badges) = harness.send_json_request(req_badges).await;
    assert_eq!(resp_badges.status(), StatusCode::OK);
    let badges = json_badges["data"]
        .as_array()
        .expect("Expected badges list");
    assert!(!badges.is_empty(), "Master badges should be auto-seeded");

    // Heartbeat of 65 seconds clears the 60s first-step threshold.
    let heartbeat_payload = serde_json::json!({
        "book_id": seeded.book1_id,
        "seconds_spent": 65
    });

    let req_heartbeat = Request::builder()
        .uri("/api/v1/activity/heartbeat")
        .method("POST")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .header("Content-Type", "application/json")
        .body(Body::from(heartbeat_payload.to_string()))
        .unwrap();

    let (resp_hb, json_hb) = harness.send_json_request(req_heartbeat).await;
    assert_eq!(resp_hb.status(), StatusCode::OK);
    assert!(
        json_hb["data"]["total_reading_seconds"]
            .as_i64()
            .unwrap_or(0)
            >= 65
    );
    assert!(json_hb["data"]["total_xp"].as_i64().unwrap_or(0) > 0);

    // A second heartbeat inside the 60 s min-interval is rejected so XP
    // cannot be farmed by replaying cheap heartbeats.
    let req_farm = Request::builder()
        .uri("/api/v1/activity/heartbeat")
        .method("POST")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .header("Content-Type", "application/json")
        .body(Body::from(heartbeat_payload.to_string()))
        .unwrap();
    let (resp_farm, json_farm) = harness.send_json_request(req_farm).await;
    assert_eq!(resp_farm.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(json_farm["error"]["code"], "RATE_LIMITED");

    // Verify user unlocked first_step badge
    let req_user_badges = Request::builder()
        .uri("/api/v1/me/badges")
        .method("GET")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .unwrap();

    let (resp_ub, json_ub) = harness.send_json_request(req_user_badges).await;
    assert_eq!(resp_ub.status(), StatusCode::OK);
    let user_badges = json_ub["data"]
        .as_array()
        .expect("Expected user badges array");
    assert!(
        !user_badges.is_empty(),
        "User should have unlocked first_step badge after 65 seconds"
    );
    assert_eq!(
        user_badges[0]["badge_id"].as_str().unwrap_or(""),
        "first_step"
    );
    harness.cleanup().await;
}

/// Heartbeat credit is clamped server-side (domain cap of 90 s per
/// heartbeat) and the anti-farm gate is per user, so alternating books
/// cannot multiply XP or streak progress.
#[tokio::test]
async fn test_heartbeat_credit_clamped_and_gate_per_user() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");
    let heartbeat = |book: Uuid, seconds: i32| {
        Request::builder()
            .uri("/api/v1/activity/heartbeat")
            .method("POST")
            .header("Authorization", format!("Bearer {}", seeded.token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({ "book_id": book, "seconds_spent": seconds }).to_string(),
            ))
            .unwrap()
    };

    let (resp, json) = harness
        .send_json_request(heartbeat(seeded.book1_id, 3600))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        json["data"]["total_reading_seconds"].as_i64(),
        Some(90),
        "a fresh reader claiming an hour is credited the 90 s cap"
    );

    let (resp, json) = harness
        .send_json_request(heartbeat(seeded.book2_id, 30))
        .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS, "{json}");

    // The profile reads the same standing back, including today's progress
    // toward the daily threshold.
    let req = Request::builder()
        .uri("/api/v1/me/streak")
        .method("GET")
        .header("Authorization", format!("Bearer {}", seeded.token))
        .body(Body::empty())
        .unwrap();
    let (resp, json) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json["data"]["total_reading_seconds"].as_i64(), Some(90));
    assert_eq!(json["data"]["today_seconds"].as_i64(), Some(90));
    assert_eq!(json["data"]["daily_threshold_seconds"].as_i64(), Some(300));
    harness.cleanup().await;
}

/// A book in `status` with one chapter, for visibility checks.
async fn seed_book_with_chapter(harness: &TestHarness, status: &str) -> (Uuid, Uuid) {
    let now = Utc::now();
    let book_id = harness.track_book(Uuid::new_v4());
    books::ActiveModel {
        id: Set(book_id),
        title: Set(format!("Visibility {status}")),
        author: Set("Hidden Author".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Misteri".to_string()),
        sub_theme: Set(None),
        description: Set(String::new()),
        cover_url: Set(String::new()),
        epub_storage_path: Set(String::new()),
        total_words: Set(100),
        estimated_reading_minutes: Set(1),
        source_name: Set("Test".to_string()),
        source_url: Set(None),
        license: Set("Private".to_string()),
        publication_year: Set(None),
        status: Set(status.to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("book seed must succeed");
    let chapter_id = Uuid::new_v4();
    chapters::ActiveModel {
        id: Set(chapter_id),
        book_id: Set(book_id),
        chapter_number: Set(1),
        title: Set("Hidden".to_string()),
        word_count: Set(100),
        html_content: Set("<p>not for readers yet</p>".to_string()),
        created_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("chapter seed must succeed");
    (book_id, chapter_id)
}

/// Chapters and insights of draft or archived books answer 404 on every
/// public path, by chapter number and by chapter UUID alike.
#[tokio::test]
async fn test_unpublished_books_hide_chapters_and_insights() {
    let harness = TestHarness::new().await;
    if harness.live_only().await.is_none() {
        panic!("live DB required (db-up)");
    }
    let (published_id, _) = seed_book_with_chapter(&harness, "published").await;
    let get = |uri: String| Request::builder().uri(uri).body(Body::empty()).unwrap();

    let resp = harness
        .send_request(get(format!("/api/v1/books/{published_id}/chapters/1")))
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "published control");

    for status in ["draft", "archived"] {
        let (book_id, chapter_id) = seed_book_with_chapter(&harness, status).await;
        let hidden = [
            format!("/api/v1/books/{book_id}/chapters/1"),
            format!("/api/v1/books/{book_id}/chapters/1/atomic-cards"),
            format!("/api/v1/books/{book_id}/chapters/{chapter_id}/atomic-cards"),
            format!("/api/v1/books/{book_id}/chapters/1/recap"),
            format!("/api/v1/books/{book_id}/chapters/{chapter_id}/recap"),
            format!("/api/v1/books/{book_id}/offline-bundle"),
        ];
        for uri in hidden {
            let (resp, body) = harness.send_json_request(get(uri.clone())).await;
            assert_eq!(
                resp.status(),
                StatusCode::NOT_FOUND,
                "{status} {uri}: {body}"
            );
            assert_eq!(body["error"]["code"], "NOT_FOUND", "{status} {uri}");
        }
    }
    harness.cleanup().await;
}

/// Progress writes on a draft book answer 404 (single and merge), and a
/// stored position on a book unpublished later stays off the shelf.
#[tokio::test]
async fn test_progress_ignores_unpublished_books() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live services required");
    let ip = common::unique_ip("198.22");
    let (token, _) = common::provision_verified_user(&harness, &mut redis, "draftprog", &ip).await;
    let (draft_id, draft_chapter) = seed_book_with_chapter(&harness, "draft").await;
    let (later_id, later_chapter) = seed_book_with_chapter(&harness, "published").await;
    let bearer = |method: &str, uri: String, body: serde_json::Value| {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    let position = |chapter: Uuid| {
        serde_json::json!({
            "chapter_id": chapter,
            "last_anchor_cfi": "p-3",
            "completion_percentage": 12.5
        })
    };

    let resp = harness
        .send_request(bearer(
            "PUT",
            format!("/api/v1/progress/{draft_id}"),
            position(draft_chapter),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let merge = serde_json::json!({ "records": [{
        "book_id": draft_id,
        "last_chapter_id": draft_chapter,
        "last_anchor_cfi": "p-3",
        "completion_percentage": 12.5,
        "last_read_at": Utc::now()
    }]});
    let resp = harness
        .send_request(bearer("POST", "/api/v1/progress/merge".to_string(), merge))
        .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let resp = harness
        .send_request(bearer(
            "PUT",
            format!("/api/v1/progress/{later_id}"),
            position(later_chapter),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    books::Entity::update_many()
        .col_expr(
            books::Column::Status,
            sea_orm::sea_query::Expr::value("archived"),
        )
        .filter(books::Column::Id.eq(later_id))
        .exec(&harness.state.db)
        .await
        .unwrap();
    let (resp, active) = harness
        .send_json_request(bearer(
            "GET",
            "/api/v1/progress/active".to_string(),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        active["data"].is_null() || active["data"]["book_id"] != serde_json::json!(later_id),
        "{active}"
    );
    harness.cleanup().await;
}

/// In-book search returns matching paragraphs in reading order with the
/// reader's `p-N` index, escapes LIKE wildcards, and hides unpublished books.
#[tokio::test]
async fn test_book_text_search_finds_paragraphs() {
    let harness = TestHarness::new().await;
    if harness.live_only().await.is_none() {
        panic!("live DB required (db-up)");
    }
    let (book_id, _) = seed_book_with_chapter(&harness, "published").await;
    // The rows the worker writes for "<h2>Pembuka</h2><p>Kabut turun di
    // pelabuhan.</p><p>Laras menulis surat &amp; menunggu kapal 100%
    // penuh.</p><p>Kapal tiba.</p><p>the secure\n   ancestral hall</p>":
    // entities decoded, whitespace collapsed, numbered like the reader's p-N.
    seed_paragraphs(
        &harness,
        book_id,
        &[
            "Kabut turun di pelabuhan.",
            "Laras menulis surat & menunggu kapal 100% penuh.",
            "Kapal tiba.",
            "the secure ancestral hall",
        ],
    )
    .await;
    let search = |q: &str| {
        Request::builder()
            .uri(format!("/api/v1/books/{book_id}/search?q={}", urlencode(q)))
            .body(Body::empty())
            .unwrap()
    };

    let (resp, body) = harness.send_json_request(search("KAPAL")).await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let hits = body["data"].as_array().unwrap();
    let anchors: Vec<i64> = hits
        .iter()
        .map(|h| h["paragraph"].as_i64().unwrap())
        .collect();
    assert_eq!(anchors, vec![2, 3], "reading order, p-N indexes");
    assert_eq!(hits[0]["chapter_number"], 1);
    assert_eq!(hits[0]["chapter_title"], "Hidden");
    assert_eq!(
        hits[0]["snippet"],
        "Laras menulis surat & menunggu kapal 100% penuh."
    );

    let (_, body) = harness.send_json_request(search("100%")).await;
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        1,
        "a literal percent sign"
    );
    let (_, body) = harness.send_json_request(search("%_")).await;
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        0,
        "wildcards stay literal"
    );

    let (_, body) = harness.send_json_request(search("secure ancestral")).await;
    assert_eq!(
        body["data"][0]["paragraph"], 4,
        "a phrase broken across source lines: {body}"
    );
    let (_, body) = harness.send_json_request(search("surat & menunggu")).await;
    assert_eq!(body["data"][0]["paragraph"], 2, "a decoded entity: {body}");

    let (resp, _) = harness.send_json_request(search("k")).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let (draft_id, _) = seed_book_with_chapter(&harness, "draft").await;
    let req = Request::builder()
        .uri(format!("/api/v1/books/{draft_id}/search?q=hidden"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        harness.send_request(req).await.status(),
        StatusCode::NOT_FOUND
    );
    harness.cleanup().await;
}

/// Writes chapter 1's paragraph index the way the worker does.
async fn seed_paragraphs(harness: &TestHarness, book_id: Uuid, texts: &[&str]) {
    use sea_orm::ConnectionTrait;
    for (n, text) in texts.iter().enumerate() {
        harness
            .state
            .db
            .execute(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "INSERT INTO chapter_paragraphs (book_id, chapter_number, n, text, folded) VALUES ($1, 1, $2, $3, lower($3))",
                [book_id.into(), (n as i32 + 1).into(), (*text).into()],
            ))
            .await
            .expect("paragraph seed must succeed");
    }
}

fn urlencode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// A draft, an archived book, and an id that never existed answer with one
/// identical 404, so the catalog is no oracle for unpublished manuscripts.
#[tokio::test]
async fn test_unpublished_and_missing_books_answer_the_same_404() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up)");
    let mut archived: books::ActiveModel = books::Entity::find_by_id(seeded.book3_id)
        .one(&harness.state.db)
        .await
        .unwrap()
        .expect("seeded book")
        .into();
    archived.status = Set("archived".to_string());
    archived.update(&harness.state.db).await.unwrap();

    let mut answers = Vec::new();
    for id in [seeded.draft_id, seeded.book3_id, Uuid::new_v4()] {
        let req = Request::builder()
            .uri(format!("/api/v1/books/{id}"))
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let (resp, json) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{json}");
        answers.push(json["error"].clone());
    }
    assert!(
        answers.iter().all(|a| *a == answers[0]),
        "bodies differ: {answers:?}"
    );
    harness.cleanup().await;
}

async fn seed_dated_book(
    harness: &TestHarness,
    theme: &str,
    year: Option<i32>,
    description: &str,
) -> Uuid {
    let now = Utc::now();
    let book_id = harness.track_book(Uuid::new_v4());
    books::ActiveModel {
        id: Set(book_id),
        title: Set(format!("Keyset {year:?}")),
        author: Set("Keyset Author".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set(theme.to_string()),
        sub_theme: Set(None),
        description: Set(description.to_string()),
        cover_url: Set(String::new()),
        epub_storage_path: Set(String::new()),
        total_words: Set(100),
        estimated_reading_minutes: Set(1),
        source_name: Set("Test".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(year),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("book seed must succeed");
    book_id
}

async fn catalog_ids(harness: &TestHarness, query: &str) -> Vec<String> {
    let req = Request::builder()
        .uri(format!("/api/v1/books?{query}"))
        .body(Body::empty())
        .expect("valid request");
    let (resp, json) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK, "{json}");
    json["data"]
        .as_array()
        .expect("data array")
        .iter()
        .map(|b| b["id"].as_str().expect("id").to_string())
        .collect()
}

/// Walking the catalog page by page reaches every published book exactly
/// once, books without a publication year included, at any page size.
#[tokio::test]
async fn test_catalog_cursor_reaches_every_book_once() {
    let harness = TestHarness::new().await;
    // A theme of its own keeps books other tests publish out of the walk.
    let theme = format!("Keyset {}", Uuid::new_v4().simple());
    for year in [
        Some(2001),
        Some(2001),
        Some(1999),
        None,
        None,
        Some(2010),
        None,
    ] {
        seed_dated_book(&harness, &theme, year, "").await;
    }
    let theme_q = urlencode(&theme);
    let everything = catalog_ids(&harness, &format!("theme={theme_q}&limit=100")).await;
    assert_eq!(everything.len(), 7, "{everything:?}");

    for limit in [1, 3] {
        let mut walked: Vec<String> = Vec::new();
        let mut cursor = String::new();
        for _ in 0..10 {
            let page =
                catalog_ids(&harness, &format!("theme={theme_q}&limit={limit}{cursor}")).await;
            let Some(last) = page.last() else { break };
            cursor = format!("&cursor={last}");
            walked.extend(page);
        }
        assert_eq!(
            walked, everything,
            "limit={limit} must walk the same order once"
        );
    }
    harness.cleanup().await;
}

/// The catalog query reads `idx_books_published_year_id` in its own order,
/// with and without a cursor, so no page pays for a sort.
#[tokio::test]
async fn test_catalog_page_plan_reads_the_index_without_a_sort() {
    use sea_orm::{ConnectionTrait, QueryTrait, Statement, TransactionTrait};
    let harness = TestHarness::new().await;
    let cursor = books::Model {
        id: Uuid::new_v4(),
        title: String::new(),
        author: String::new(),
        language: "en".to_string(),
        primary_theme: String::new(),
        sub_theme: None,
        description: String::new(),
        cover_url: String::new(),
        epub_storage_path: String::new(),
        total_words: 0,
        estimated_reading_minutes: 0,
        source_name: String::new(),
        source_url: None,
        license: String::new(),
        publication_year: Some(1950),
        status: "published".to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };
    let undated = books::Model {
        publication_year: None,
        ..cursor.clone()
    };
    let backend = harness.state.db.get_database_backend();
    for page_after in [None, Some(&cursor), Some(&undated)] {
        let published = books::Entity::find().filter(books::Column::Status.eq("published"));
        let sql = infra::repositories::book_repository::catalog_page(published, page_after, 20)
            .build(backend)
            .to_string();
        let txn = harness.state.db.begin().await.expect("begin");
        // A small dev table makes a sort cheap; with sorting penalised, a plan
        // still sorts only when the order is one the index cannot give.
        for switch in ["enable_seqscan", "enable_bitmapscan", "enable_sort"] {
            txn.execute(Statement::from_string(
                backend,
                format!("SET LOCAL {switch} = OFF"),
            ))
            .await
            .expect("planner switch");
        }
        let plan: String = txn
            .query_all(Statement::from_string(backend, format!("EXPLAIN {sql}")))
            .await
            .expect("EXPLAIN")
            .iter()
            .filter_map(|row| row.try_get_by_index::<String>(0).ok())
            .collect::<Vec<_>>()
            .join("\n");
        txn.rollback().await.expect("rollback");
        assert!(
            plan.contains("idx_books_published_year_id"),
            "{sql}\n{plan}"
        );
        assert!(!plan.contains("Sort"), "{sql}\n{plan}");
    }
    harness.cleanup().await;
}

/// Words only the description holds are found: the query is parsed with
/// the same `simple` config the index uses, not the database's `english`.
#[tokio::test]
async fn test_catalog_search_finds_words_only_in_the_description() {
    let harness = TestHarness::new().await;
    let theme = format!("Keyset {}", Uuid::new_v4().simple());
    let book_id =
        seed_dated_book(&harness, &theme, None, "The running stories of the harbour").await;
    let req = Request::builder()
        .uri("/api/v1/books/search?q=running%20stories")
        .body(Body::empty())
        .expect("valid request");
    let (resp, json) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK, "{json}");
    let ids: Vec<&str> = json["data"]
        .as_array()
        .expect("data array")
        .iter()
        .filter_map(|b| b["id"].as_str())
        .collect();
    assert!(ids.contains(&book_id.to_string().as_str()), "{json}");
    harness.cleanup().await;
}

async fn seed_reader(harness: &TestHarness) -> (Uuid, String) {
    let now = Utc::now();
    let user_id = harness.track_user(Uuid::new_v4());
    let email = format!("tz_{}@baca.local", Uuid::new_v4());
    users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(Some("test_password_hash".to_string())),
        display_name: Set("Pembaca WIB".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&harness.state.db)
    .await
    .expect("user seed must succeed");
    let token = infra::generate_access_token(
        user_id,
        &email,
        "reader",
        harness.state.config.jwt_secret(),
        1440,
    )
    .expect("token");
    (user_id, token)
}

async fn heartbeat(
    harness: &TestHarness,
    token: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .uri("/api/v1/activity/heartbeat")
        .method("POST")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("valid request");
    let (resp, json) = harness.send_json_request(req).await;
    (resp.status(), json)
}

/// A streak day is the reader's own: reading is logged on their calendar
/// day, a reader whose local yesterday counted reaches day two today, and an
/// offset no clock uses is refused.
#[tokio::test]
async fn test_streak_days_follow_the_readers_clock() {
    use infra::entities::{reading_activity_logs, user_reading_streaks};
    use sea_orm::QueryOrder;
    let harness = TestHarness::new().await;
    let book = common::seed_book(&harness, "published").await;
    harness.track_book(book);
    let db = &harness.state.db;

    // The same instant is a different calendar day at UTC+14 and UTC-12.
    for offset in [840, -720] {
        let (user, token) = seed_reader(&harness).await;
        let (status, json) = heartbeat(
            &harness,
            &token,
            serde_json::json!({"book_id": book, "seconds_spent": 30, "utc_offset_minutes": offset}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{json}");
        let logged = reading_activity_logs::Entity::find()
            .filter(reading_activity_logs::Column::UserId.eq(user))
            .order_by_desc(reading_activity_logs::Column::CreatedAt)
            .one(db)
            .await
            .expect("query")
            .expect("logged");
        let expected = domain::local_date(logged.created_at.with_timezone(&Utc), offset);
        assert_eq!(logged.activity_date, expected, "offset {offset}");
    }

    // Read 23:00 yesterday (WIB), now 06:30: the second day of a streak.
    let (user, token) = seed_reader(&harness).await;
    let today = domain::local_date(Utc::now(), 420);
    let yesterday = today.pred_opt().expect("a day before");
    user_reading_streaks::ActiveModel {
        user_id: Set(user),
        current_streak_days: Set(1),
        longest_streak_days: Set(1),
        total_reading_seconds: Set(300),
        total_xp: Set(0),
        last_activity_date: Set(Some(yesterday)),
        updated_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await
    .expect("streak seed");
    reading_activity_logs::ActiveModel {
        user_id: Set(user),
        book_id: Set(book),
        seconds_spent: Set(290),
        activity_date: Set(today),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("log seed");
    let (status, json) = heartbeat(
        &harness,
        &token,
        serde_json::json!({"book_id": book, "seconds_spent": 30, "utc_offset_minutes": 420}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["data"]["current_streak_days"], 2, "{json}");

    let req = Request::builder()
        .uri("/api/v1/me/streak?utc_offset_minutes=420")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("valid request");
    let (resp, streak) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK, "{streak}");
    assert_eq!(streak["data"]["today_seconds"], 320, "{streak}");

    let (_, token) = seed_reader(&harness).await;
    let (status, json) = heartbeat(
        &harness,
        &token,
        serde_json::json!({"book_id": book, "seconds_spent": 30, "utc_offset_minutes": 9999}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{json}");
    assert_eq!(json["error"]["code"], "VALIDATION_FAILED");
    harness.cleanup().await;
}
