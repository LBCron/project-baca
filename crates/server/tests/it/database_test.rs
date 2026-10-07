//! Constraint, cascade, rollback, and query-plan tests.

use crate::common;

use chrono::Utc;
use common::TestHarness;
use infra::entities::{
    book_tags, books, chapters, tags, user_badges, user_reading_progress, users,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, Statement,
    TransactionTrait,
};
use uuid::Uuid;

#[tokio::test]
async fn test_db_unique_email_constraint_enforcement() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let shared_email = format!("dup_{}@example.com", Uuid::new_v4());
    let now = Utc::now();

    // Insert first user
    let user1 = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(shared_email.clone()),
        password_hash: Set(Some("hash1".to_string())),
        display_name: Set("User One".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    user1
        .insert(&harness.state.db)
        .await
        .expect("First user insert should succeed");

    // Attempt to insert second user with identical email
    let user2 = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(shared_email.clone()),
        password_hash: Set(Some("hash2".to_string())),
        display_name: Set("User Two".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    let insert_result = user2.insert(&harness.state.db).await;

    assert!(
        insert_result.is_err(),
        "Database must reject duplicate email insertion"
    );
    let err_str = insert_result.unwrap_err().to_string();
    assert!(
        err_str.contains("unique") || err_str.contains("duplicate"),
        "Error message should indicate unique constraint violation: {err_str}"
    );
}

#[tokio::test]
async fn test_db_user_role_check_constraint() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let now = Utc::now();
    let invalid_user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(format!("invalid_role_{}@example.com", Uuid::new_v4())),
        password_hash: Set(Some("hash".to_string())),
        display_name: Set("Invalid Role User".to_string()),
        role: Set("superuser_invalid".to_string()), // violates CHECK (role IN ('reader', 'admin'))
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    let result = invalid_user.insert(&harness.state.db).await;
    assert!(
        result.is_err(),
        "Database check constraint must reject unauthorized user roles"
    );
}

#[tokio::test]
async fn test_db_unique_tag_slug_constraint() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let shared_slug = format!("slug-{}", Uuid::new_v4());
    let now = Utc::now();

    let tag1 = tags::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(format!("Tag Name {}", Uuid::new_v4())),
        slug: Set(shared_slug.clone()),
        created_at: Set(now.into()),
    };
    tag1.insert(&harness.state.db)
        .await
        .expect("Tag 1 insert should succeed");

    let tag2 = tags::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(format!("Tag Name Other {}", Uuid::new_v4())),
        slug: Set(shared_slug),
        created_at: Set(now.into()),
    };
    let result = tag2.insert(&harness.state.db).await;
    assert!(result.is_err(), "Duplicate tag slug must be rejected");
}

#[tokio::test]
async fn test_db_foreign_key_cascade_on_user_deletion() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let now = Utc::now();
    let user_id = Uuid::new_v4();

    // Create User
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("cascade_u_{}@example.com", user_id)),
        password_hash: Set(Some("hash".to_string())),
        display_name: Set("Cascade User".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    user.insert(&harness.state.db)
        .await
        .expect("User insert failed");

    // Create Book & Chapter
    let book_id = Uuid::new_v4();
    let book = books::ActiveModel {
        id: Set(book_id),
        title: Set("Cascade Test Novel".to_string()),
        author: Set("Classic Writer".to_string()),
        language: Set("id".to_string()),
        primary_theme: Set("Fiction".to_string()),
        sub_theme: Set(None),
        description: Set("A novel for testing cascades".to_string()),
        cover_url: Set("/covers/test.jpg".to_string()),
        epub_storage_path: Set("/epubs/test.epub".to_string()),
        total_words: Set(10000),
        estimated_reading_minutes: Set(60),
        source_name: Set("Test Source".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1925)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    book.insert(&harness.state.db)
        .await
        .expect("Book insert failed");

    let chapter_id = Uuid::new_v4();
    let chapter = chapters::ActiveModel {
        id: Set(chapter_id),
        book_id: Set(book_id),
        chapter_number: Set(1),
        title: Set("Chapter 1".to_string()),
        word_count: Set(2500),
        html_content: Set("<p>Chapter content</p>".to_string()),
        created_at: Set(now.into()),
    };
    chapter
        .insert(&harness.state.db)
        .await
        .expect("Chapter insert failed");

    // Create Reading Progress for User
    let progress_id = Uuid::new_v4();
    let progress = user_reading_progress::ActiveModel {
        id: Set(progress_id),
        user_id: Set(user_id),
        book_id: Set(book_id),
        last_chapter_id: Set(chapter_id),
        last_anchor_cfi: Set("epubcfi(/6/2[chap1]!/4/2)".to_string()),
        completion_percentage: Set(sea_orm::prelude::Decimal::new(355, 1)),
        is_finished: Set(false),
        last_read_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    progress
        .insert(&harness.state.db)
        .await
        .expect("Progress insert failed");

    // Create User Badge
    infra::seed_default_badges_if_empty(&harness.state.db)
        .await
        .expect("Badge seed failed");
    let user_badge = user_badges::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        badge_id: Set("first_step".to_string()),
        unlocked_at: Set(now.into()),
    };
    user_badge
        .insert(&harness.state.db)
        .await
        .expect("User badge insert failed");

    // Verify records exist before deletion
    let progress_count_before = user_reading_progress::Entity::find()
        .filter(user_reading_progress::Column::UserId.eq(user_id))
        .all(&harness.state.db)
        .await
        .expect("Query progress failed")
        .len();
    assert_eq!(progress_count_before, 1);

    // Delete User (Triggers PostgreSQL Foreign Key CASCADE)
    users::Entity::delete_by_id(user_id)
        .exec(&harness.state.db)
        .await
        .expect("User deletion failed");

    // Verify cascading deletion cleaned up progress and badges
    let progress_after = user_reading_progress::Entity::find()
        .filter(user_reading_progress::Column::UserId.eq(user_id))
        .all(&harness.state.db)
        .await
        .expect("Query progress after deletion failed");
    assert_eq!(
        progress_after.len(),
        0,
        "User progress must be cascaded on user deletion"
    );

    let badges_after = user_badges::Entity::find()
        .filter(user_badges::Column::UserId.eq(user_id))
        .all(&harness.state.db)
        .await
        .expect("Query badges after deletion failed");
    assert_eq!(
        badges_after.len(),
        0,
        "User badges must be cascaded on user deletion"
    );
}

#[tokio::test]
async fn test_db_foreign_key_cascade_on_book_deletion() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let now = Utc::now();
    let book_id = Uuid::new_v4();

    // Insert Book
    let book = books::ActiveModel {
        id: Set(book_id),
        title: Set("Book For Cascade Test".to_string()),
        author: Set("Author Name".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Historical".to_string()),
        sub_theme: Set(None),
        description: Set("Description".to_string()),
        cover_url: Set("/cover.png".to_string()),
        epub_storage_path: Set("/book.epub".to_string()),
        total_words: Set(5000),
        estimated_reading_minutes: Set(30),
        source_name: Set("Project Gutenberg".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1910)),
        status: Set("published".to_string()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    book.insert(&harness.state.db)
        .await
        .expect("Book insert failed");

    // Insert Chapter
    let chapter = chapters::ActiveModel {
        id: Set(Uuid::new_v4()),
        book_id: Set(book_id),
        chapter_number: Set(1),
        title: Set("Chapter 1".to_string()),
        word_count: Set(1500),
        html_content: Set("<p>Chapter</p>".to_string()),
        created_at: Set(now.into()),
    };
    chapter
        .insert(&harness.state.db)
        .await
        .expect("Chapter insert failed");

    // Insert Tag and BookTag
    let tag_id = Uuid::new_v4();
    let tag = tags::ActiveModel {
        id: Set(tag_id),
        name: Set(format!("Tag-{}", tag_id)),
        slug: Set(format!("slug-{}", tag_id)),
        created_at: Set(now.into()),
    };
    tag.insert(&harness.state.db)
        .await
        .expect("Tag insert failed");

    let book_tag = book_tags::ActiveModel {
        book_id: Set(book_id),
        tag_id: Set(tag_id),
    };
    book_tag
        .insert(&harness.state.db)
        .await
        .expect("BookTag insert failed");

    // Delete Book
    books::Entity::delete_by_id(book_id)
        .exec(&harness.state.db)
        .await
        .expect("Book deletion failed");

    // Verify chapters and book_tags are cascaded
    let chapters_after = chapters::Entity::find()
        .filter(chapters::Column::BookId.eq(book_id))
        .all(&harness.state.db)
        .await
        .expect("Query chapters failed");
    assert_eq!(
        chapters_after.len(),
        0,
        "Chapters must be cascaded on book deletion"
    );

    let book_tags_after = book_tags::Entity::find()
        .filter(book_tags::Column::BookId.eq(book_id))
        .all(&harness.state.db)
        .await
        .expect("Query book_tags failed");
    assert_eq!(
        book_tags_after.len(),
        0,
        "Book tags must be cascaded on book deletion"
    );
}

#[tokio::test]
async fn test_db_transaction_atomicity_and_rollback() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let user_id = Uuid::new_v4();
    let test_email = format!("rollback_{}@example.com", user_id);
    let now = Utc::now();

    // Start a transaction
    let txn = harness
        .state
        .db
        .begin()
        .await
        .expect("Failed to begin database transaction");

    // Insert user inside transaction
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(test_email.clone()),
        password_hash: Set(Some("hash".to_string())),
        display_name: Set("Rollback Test User".to_string()),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    user.insert(&txn)
        .await
        .expect("User insert inside txn should succeed");

    // Rollback explicitly
    txn.rollback()
        .await
        .expect("Transaction rollback should succeed");

    // Verify user record does NOT exist in main database connection
    let found = users::Entity::find_by_id(user_id)
        .one(&harness.state.db)
        .await
        .expect("Query after rollback failed");

    assert!(
        found.is_none(),
        "Record inserted inside rolled-back transaction must not be persisted"
    );
}

#[tokio::test]
async fn test_db_index_query_plan_verification() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    // Verify index registration in PostgreSQL system catalog
    let catalog_check = Statement::from_string(
        harness.state.db.get_database_backend(),
        "SELECT indexname FROM pg_indexes WHERE tablename = 'users' AND indexname = 'idx_users_email';",
    );
    let rows = harness
        .state
        .db
        .query_all(catalog_check)
        .await
        .expect("Query pg_indexes failed");
    assert!(
        !rows.is_empty(),
        "Index 'idx_users_email' must exist in PostgreSQL catalog"
    );

    // Begin transaction and disable seqscan to verify index path usability
    let txn = harness.state.db.begin().await.expect("Begin txn failed");
    txn.execute(Statement::from_string(
        harness.state.db.get_database_backend(),
        "SET LOCAL enable_seqscan = OFF;",
    ))
    .await
    .expect("Set enable_seqscan failed");

    let explain_stmt = Statement::from_string(
        harness.state.db.get_database_backend(),
        "EXPLAIN (FORMAT TEXT) SELECT * FROM users WHERE email = 'benchmark@test.local';",
    );
    let query_result = txn
        .query_all(explain_stmt)
        .await
        .expect("EXPLAIN query failed");

    let mut full_plan = String::new();
    for row in query_result {
        if let Ok(line) = row.try_get_by_index::<String>(0) {
            full_plan.push_str(&line);
            full_plan.push('\n');
        }
    }

    assert!(
        full_plan.contains("Index Scan") || full_plan.contains("Bitmap Index Scan"),
        "PostgreSQL query planner should utilize index for email lookups. Plan:\n{full_plan}"
    );
    assert!(
        full_plan.contains("idx_users_email"),
        "Query plan must explicitly use 'idx_users_email'. Plan:\n{full_plan}"
    );
}

#[tokio::test]
async fn test_db_catalog_default_pagination_index_plan() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    // Begin transaction and disable seqscan to verify index path usability
    let txn = harness.state.db.begin().await.expect("Begin txn failed");
    txn.execute(Statement::from_string(
        harness.state.db.get_database_backend(),
        "SET LOCAL enable_seqscan = OFF;",
    ))
    .await
    .expect("Set enable_seqscan failed");

    let explain_stmt = Statement::from_string(
        harness.state.db.get_database_backend(),
        "EXPLAIN (FORMAT TEXT) SELECT id, title, author, publication_year FROM books WHERE status = 'published' ORDER BY publication_year DESC NULLS LAST, id ASC LIMIT 20;",
    );
    let query_result = txn
        .query_all(explain_stmt)
        .await
        .expect("EXPLAIN query failed");

    let mut full_plan = String::new();
    for row in query_result {
        if let Ok(line) = row.try_get_by_index::<String>(0) {
            full_plan.push_str(&line);
            full_plan.push('\n');
        }
    }

    assert!(
        full_plan.contains("idx_books_published_year_id"),
        "Default catalog pagination must use 'idx_books_published_year_id'. Plan:\n{full_plan}"
    );
    assert!(
        !full_plan.contains("Sort"),
        "Default catalog pagination must avoid an in-memory Sort node. Plan:\n{full_plan}"
    );
}

#[tokio::test]
async fn test_db_foreign_key_and_optimized_index_coverage() {
    let harness = TestHarness::new().await;
    let _ = harness.live_only().await.expect("live DB required (db-up)");

    let required_indexes = [
        "idx_book_chunks_chapter_id",
        "idx_user_progress_book_id",
        "idx_user_progress_chapter_id",
        "idx_reading_activity_book_id",
        "idx_saved_quotes_book_id",
        "idx_saved_quotes_chapter_id",
        "idx_user_badges_badge_id",
        "idx_tldr_cache_chapter_id",
        "idx_books_published_year_id",
    ];

    for idx_name in required_indexes {
        let query = Statement::from_string(
            harness.state.db.get_database_backend(),
            format!("SELECT indexname FROM pg_indexes WHERE indexname = '{idx_name}';"),
        );
        let rows = harness
            .state
            .db
            .query_all(query)
            .await
            .expect("Query pg_indexes failed");
        assert!(
            !rows.is_empty(),
            "Required performance index '{idx_name}' must exist in PostgreSQL catalog"
        );
    }
}
