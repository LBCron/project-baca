//! Catalog, search, reader, and gamification tests.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use common::TestHarness;
use infra::entities::{book_tags, books, chapters, tags, users};
use sea_orm::{ActiveModelTrait, Set};
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
        .expect("live DB required (db-up); 11c gate");

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

    // Test SRS top-level alias (/api/books)
    let req_alias = Request::builder()
        .uri("/api/books?language=id")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let (resp_alias, json_alias) = harness.send_json_request(req_alias).await;
    assert_eq!(resp_alias.status(), StatusCode::OK);
    assert!(json_alias["success"].as_bool().unwrap_or(false));
}

#[tokio::test]
async fn test_catalog_typo_tolerant_fts_search() {
    let harness = TestHarness::new().await;
    let _seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up); 11c gate");

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
}

#[tokio::test]
async fn test_book_overview_chapter_and_offline_bundle() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up); 11c gate");

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
}

#[tokio::test]
async fn test_reading_progress_and_active_retrieval() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up); 11c gate");

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
}

#[tokio::test]
async fn test_gamification_heartbeat_and_badges() {
    let harness = TestHarness::new().await;
    let seeded = seed_test_catalog(&harness)
        .await
        .expect("live DB required (db-up); 11c gate");

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
}
