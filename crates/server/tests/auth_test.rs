//! Auth lifecycle tests: signup, OTP, login, rotation, logout.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestHarness;
use pretty_assertions::assert_eq;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Statement};
use shared::{
    ChangePasswordRequest, GuestMergeRequest, GuestProgressRecord, LoginRequest,
    RefreshTokenRequest, SignupRequest, UpdateProfileRequest, VerifyOtpRequest,
};
use uuid::Uuid;

#[tokio::test]
async fn test_auth_full_lifecycle() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.state.get_redis_conn().await {
        Ok(c) => c,
        Err(_) => {
            println!("Redis not reachable, skipping full integration flow");
            return;
        }
    };

    // A unique IP per run keeps this flow out of the shared rate-limit
    // bucket (the lifecycle alone issues ~10 auth requests).
    let lifecycle_ip = format!(
        "198.51.{}.{}",
        200 + (Uuid::new_v4().as_u128() % 40) as u8,
        200
    );

    let test_email = format!("reader_{}@example.com", Uuid::new_v4());
    let test_password = "Password1234!";

    // Signup
    let signup_req = SignupRequest {
        display_name: "VintageReader".to_string(),
        email: test_email.clone(),
        password: test_password.to_string(),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(body["success"], true);

    // Fetch OTP from Redis
    let redis_key = format!("otp:{test_email}");
    let record_str: Option<String> = redis_conn.get(&redis_key).await.unwrap();
    assert!(record_str.is_some(), "OTP record should exist in Redis");

    // We can directly verify that a wrong OTP fails
    let wrong_otp_req = VerifyOtpRequest {
        email: test_email.clone(),
        otp: "000000".to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&wrong_otp_req).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Let's set a known OTP directly in Redis to test valid verification
    let known_otp = "889911";
    let known_hash = infra::hash_otp(known_otp);
    let record_json = serde_json::json!({ "hash": known_hash, "attempts": 0 }).to_string();
    let _: () = redis_conn
        .set_ex(&redis_key, record_json, 600)
        .await
        .unwrap();

    // Verify OTP with known valid OTP
    let verify_req = VerifyOtpRequest {
        email: test_email.clone(),
        otp: known_otp.to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&verify_req).unwrap()))
        .unwrap();
    let (resp, verify_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(verify_body["success"], true);

    let access_token = verify_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    let refresh_token = verify_body["data"]["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());

    // Login with credentials
    let login_req = LoginRequest {
        email: test_email.clone(),
        password: test_password.to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_req).unwrap()))
        .unwrap();
    let (resp, login_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let active_access_token = login_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    let active_refresh_token = login_body["data"]["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Get Profile (/api/v1/me)
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {active_access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, me_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(me_body["data"]["display_name"], "VintageReader");
    assert_eq!(me_body["data"]["email"], test_email);

    // Update Profile (/api/v1/me)
    let update_req = UpdateProfileRequest {
        display_name: Some("MasterBibliophile".to_string()),
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
    };
    let req = Request::builder()
        .method("PATCH")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {active_access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&update_req).unwrap()))
        .unwrap();
    let (resp, updated_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(updated_body["data"]["display_name"], "MasterBibliophile");

    // Change Password (/api/v1/me/password) with explicit opt-out of session
    // revocation so the pre-change refresh token stays valid for the
    // rotation assertions below.
    let new_password = "BrandNewSecurePassword123!";
    let change_pwd_req = ChangePasswordRequest {
        current_password: test_password.to_string(),
        new_password: new_password.to_string(),
        revoke_other_sessions: Some(false),
    };
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/password")
        .header("authorization", format!("Bearer {active_access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&change_pwd_req).unwrap()))
        .unwrap();
    let (resp, pwd_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(pwd_body["success"], true);

    // Refresh Token Rotation
    let refresh_req = RefreshTokenRequest {
        refresh_token: active_refresh_token.clone(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&refresh_req).unwrap()))
        .unwrap();
    let (resp, rotated_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let rotated_refresh_token = rotated_body["data"]["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(
        active_refresh_token, rotated_refresh_token,
        "Token must be rotated"
    );

    // Old refresh token must be invalidated
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&refresh_req).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Replaying the rotated token again triggers theft handling: the family
    // is revoked, so even the freshly rotated token dies.
    let replay_req = RefreshTokenRequest {
        refresh_token: refresh_req.refresh_token.clone(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&replay_req).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&RefreshTokenRequest {
                refresh_token: rotated_refresh_token.clone(),
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // The theft killed the family: re-login for the logout + delete steps.
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&LoginRequest {
                email: test_email.clone(),
                password: new_password.to_string(),
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, relogin_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let rotated_refresh_token = relogin_body["data"]["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();
    let active_access_token = relogin_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Logout with rotated token
    let logout_req = RefreshTokenRequest {
        refresh_token: rotated_refresh_token,
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {active_access_token}"))
        .body(Body::from(serde_json::to_vec(&logout_req).unwrap()))
        .unwrap();
    let (resp, logout_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(logout_body["success"], true);

    // The access token used at logout is now blacklisted.
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {active_access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Re-login for the account-deletion step (the old token is revoked).
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("cf-connecting-ip", &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&LoginRequest {
                email: test_email.clone(),
                password: new_password.to_string(),
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, relogin_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let fresh_access_token = relogin_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete Account
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {fresh_access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, delete_body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(delete_body["success"], true);
}

#[tokio::test]
async fn test_change_password_revokes_sessions_by_default() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4();
    let email = format!("pwd_revoke_{user_id}@example.com");
    let user = infra::entities::users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        display_name: Set("PwdRevoker".to_string()),
        password_hash: Set(None),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    if user.insert(&harness.state.db).await.is_err() {
        println!("Database not reachable, skipping password-revoke test");
        return;
    }

    let password = "OrigSecurePassword123!";
    let stored = infra::hash_password_async(password.to_string())
        .await
        .unwrap();
    infra::update_user_password(&harness.state.db, user_id, &stored)
        .await
        .unwrap();
    let access_token = infra::generate_access_token(
        user_id,
        &email,
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .unwrap();

    // Change password without the flag: other sessions must be revoked.
    let change_req = ChangePasswordRequest {
        current_password: password.to_string(),
        new_password: "BrandNewSecurePassword456!".to_string(),
        revoke_other_sessions: None,
    };
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/password")
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&change_req).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // The pre-change access token is now revoked via user_revoked_before.
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_guest_progress_merge() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4();
    let email = format!("guest_merger_{}@example.com", user_id);

    // Create user in DB directly
    let user = infra::entities::users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        display_name: Set("GuestReader".to_string()),
        password_hash: Set(Some("hash".to_string())),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    if user.insert(&harness.state.db).await.is_err() {
        println!("Database not reachable, skipping merge test");
        return;
    }

    // Seed test book & chapter
    let book_id = Uuid::new_v4();
    let chapter_id = Uuid::new_v4();
    let book = infra::entities::books::ActiveModel {
        id: Set(book_id),
        title: Set("Classic Novel".to_string()),
        author: Set("Jane Austen".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Fiction".to_string()),
        sub_theme: Set(None),
        description: Set("A wonderful classic novel.".to_string()),
        cover_url: Set("https://example.com/cover.jpg".to_string()),
        epub_storage_path: Set("books/novel.epub".to_string()),
        total_words: Set(50000),
        estimated_reading_minutes: Set(200),
        source_name: Set("Standard Ebooks".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(Some(1813)),
        status: Set("published".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    book.insert(&harness.state.db).await.unwrap();

    let chapter = infra::entities::chapters::ActiveModel {
        id: Set(chapter_id),
        book_id: Set(book_id),
        chapter_number: Set(1),
        title: Set("Chapter 1".to_string()),
        word_count: Set(2500),
        html_content: Set("<p>Chapter content here.</p>".to_string()),
        created_at: Set(chrono::Utc::now().into()),
    };
    chapter.insert(&harness.state.db).await.unwrap();

    let access_token = infra::generate_access_token(
        user_id,
        &email,
        "reader",
        harness.state.config.jwt_secret(),
        15,
    )
    .unwrap();

    // Initial merge at 35%
    let merge_req = GuestMergeRequest {
        records: vec![GuestProgressRecord {
            book_id,
            last_chapter_id: chapter_id,
            last_anchor_cfi: "epubcfi(/6/2[chapter1]!/4/2/10:5)".to_string(),
            completion_percentage: 35.0,
            is_finished: Some(false),
            last_read_at: Some(chrono::Utc::now()),
        }],
    };

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/progress/merge")
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&merge_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["data"]["merged_count"], 1);

    // Subsequent merge at 75% — verify GREATEST progression
    let merge_req_2 = GuestMergeRequest {
        records: vec![GuestProgressRecord {
            book_id,
            last_chapter_id: chapter_id,
            last_anchor_cfi: "epubcfi(/6/2[chapter1]!/4/2/40:15)".to_string(),
            completion_percentage: 75.0,
            is_finished: Some(false),
            last_read_at: Some(chrono::Utc::now()),
        }],
    };

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/progress/merge")
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&merge_req_2).unwrap()))
        .unwrap();

    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify database record has 75.00
    let progress_res = harness
        .state
        .db
        .query_one(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT completion_percentage FROM user_reading_progress WHERE user_id = $1 AND book_id = $2",
            vec![user_id.into(), book_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();

    let completion: sea_orm::entity::prelude::Decimal =
        progress_res.try_get_by("completion_percentage").unwrap();
    let completion_f64: f64 = completion.to_string().parse().unwrap();
    assert_eq!(completion_f64, 75.0);

    // Unknown book ids fail the whole batch with 404 and commit nothing.
    let bad_merge = GuestMergeRequest {
        records: vec![GuestProgressRecord {
            book_id: Uuid::new_v4(),
            last_chapter_id: chapter_id,
            last_anchor_cfi: "epubcfi(/6/2[chapter1]!/4/2/40:15)".to_string(),
            completion_percentage: 90.0,
            is_finished: Some(false),
            last_read_at: Some(chrono::Utc::now()),
        }],
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/progress/merge")
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&bad_merge).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cross-book chapter ids are a 400, not a 500 FK violation.
    let other_book_id = Uuid::new_v4();
    let other_book = infra::entities::books::ActiveModel {
        id: Set(other_book_id),
        title: Set("Other Novel".to_string()),
        author: Set("Someone".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Fiction".to_string()),
        sub_theme: Set(None),
        description: Set("Other.".to_string()),
        cover_url: Set(String::new()),
        epub_storage_path: Set("books/other.epub".to_string()),
        total_words: Set(1000),
        estimated_reading_minutes: Set(5),
        source_name: Set("Test".to_string()),
        source_url: Set(None),
        license: Set("Public Domain".to_string()),
        publication_year: Set(None),
        status: Set("published".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    other_book.insert(&harness.state.db).await.unwrap();
    let mismatch_merge = GuestMergeRequest {
        records: vec![GuestProgressRecord {
            book_id: other_book_id,
            last_chapter_id: chapter_id,
            last_anchor_cfi: "epubcfi(/6/2[chapter1]!/4/2/40:15)".to_string(),
            completion_percentage: 90.0,
            is_finished: Some(false),
            last_read_at: Some(chrono::Utc::now()),
        }],
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/progress/merge")
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&mismatch_merge).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Clean up
    let _ = harness
        .state
        .db
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "DELETE FROM users WHERE id = $1",
            vec![user_id.into()],
        ))
        .await;
    let _ = harness
        .state
        .db
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "DELETE FROM books WHERE id = $1",
            vec![book_id.into()],
        ))
        .await;
}
