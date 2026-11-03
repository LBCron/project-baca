//! Auth lifecycle, adversarial replay/lockout/OTP/revoke tests.

use crate::common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{provision_verified_user, TestHarness};
use pretty_assertions::assert_eq;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Statement};
use serde_json::Value;
use shared::{
    ChangePasswordRequest, GuestMergeRequest, GuestProgressRecord, LoginRequest,
    RefreshTokenRequest, SignupRequest, UpdateProfileRequest, VerifyOtpRequest,
};
use uuid::Uuid;

#[tokio::test]
async fn test_auth_full_lifecycle() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up)");

    // A unique IP per run keeps this flow out of the shared rate-limit
    // bucket (the lifecycle alone issues ~10 auth requests).
    let lifecycle_ip = format!(
        "198.51.{}.{}",
        200 + (Uuid::new_v4().as_u128() % 40) as u8,
        200
    );

    let test_email = format!("reader_{}@example.com", Uuid::new_v4());
    harness.track_email(&test_email);
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
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();

    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(body["success"], true);
    let registration_id = common::registration_of(&body);

    let record_str: Option<String> = redis_conn
        .get(infra::registration_key(&test_email, registration_id))
        .await
        .unwrap();
    assert!(record_str.is_some(), "the registration waits in Redis");
    let unverified = infra::find_user_by_email(&harness.state.db, &test_email)
        .await
        .unwrap();
    assert!(
        unverified.is_none(),
        "no account row before the code comes back"
    );

    // We can directly verify that a wrong OTP fails
    let wrong_otp_req = VerifyOtpRequest {
        email: test_email.clone(),
        registration_id,
        otp: "000000".to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&wrong_otp_req).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let known_otp = "889911";
    common::plant_known_otp(&mut redis_conn, &test_email, registration_id, known_otp).await;

    // Verify OTP with known valid OTP
    let verify_req = VerifyOtpRequest {
        email: test_email.clone(),
        registration_id,
        otp: known_otp.to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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

    // Expire the rotation grace window so the replays below read as theft
    // rather than as a second tab asking for the same successor.
    let mut redis_conn = harness
        .state
        .get_redis_conn()
        .await
        .expect("live Redis required (db-up)");
    let _: () = redis_conn
        .del(infra::rotation_grace_key(&active_refresh_token))
        .await
        .unwrap();

    // Old refresh token must be invalidated
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&replay_req).unwrap()))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
    // Sleep past the 1-second JWT iat granularity: the theft-triggered
    // family revocation above stamps user_revoked_before at whole-second
    // precision, so a token minted in the same second would compare as
    // revoked. Production clients transparently retry once.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(common::CLIENT_IP_HEADER, &lifecycle_ip)
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
    harness.cleanup().await;
}

#[tokio::test]
async fn test_change_password_revokes_sessions_by_default() {
    let harness = TestHarness::new().await;
    let user_id = harness.track_user(Uuid::new_v4());
    let email = format!("pwd_revoke_{user_id}@example.com");
    harness.track_email(&email);
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
    user.insert(&harness.state.db)
        .await
        .expect("live DB required (db-up)");

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
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let fresh_access = body["data"]["tokens"]["access_token"]
        .as_str()
        .expect("revocation hands back a fresh pair")
        .to_string();

    // The pre-change access token is now revoked via user_revoked_before.
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // The caller continues on the fresh pair without a second login.
    wait_next_second().await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {fresh_access}"))
        .body(Body::empty())
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    harness.cleanup().await;
}

#[tokio::test]
async fn test_auth_guest_progress_merge() {
    let harness = TestHarness::new().await;
    let user_id = harness.track_user(Uuid::new_v4());
    let email = format!("guest_merger_{}@example.com", user_id);
    harness.track_email(&email);

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
    user.insert(&harness.state.db)
        .await
        .expect("live DB required (db-up)");

    // Seed test book & chapter
    let book_id = harness.track_book(Uuid::new_v4());
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
    let other_book_id = harness.track_book(Uuid::new_v4());
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
    harness.cleanup().await;
}

// Adversarial tests need live Postgres + Redis (`#[ignore]` by default,
// run with `BACA_LIVE_TEST=1`). Unique email + TEST-NET IP per test keeps
// rate-limit and lockout buckets isolated across parallel runs.

/// Sleeps until the wall clock crosses into the next whole second.
/// Needed because revocation compares whole-second `iat <= revoked_before`.
async fn wait_next_second() {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let remain = 1000 - (now_ms % 1000);
    tokio::time::sleep(std::time::Duration::from_millis((remain + 50) as u64)).await;
}

fn login_req(email: &str, password: &str, ip: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(common::CLIENT_IP_HEADER, ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&LoginRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .unwrap(),
        ))
        .unwrap()
}

/// Refresh-token replay kills the whole family (theft handling):
/// after replaying a rotated token, even the *fresh* sibling must be dead.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_replay_kills_token_family() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let (_, refresh) = provision_verified_user(&harness, &mut redis_conn, "replay", &ip).await;

    let rotate = |token: &str| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/refresh")
            .header(common::CLIENT_IP_HEADER, &ip)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&RefreshTokenRequest {
                    refresh_token: token.to_string(),
                })
                .unwrap(),
            ))
            .unwrap()
    };

    let (resp, body) = harness.send_json_request(rotate(&refresh)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let fresh: String = body["data"]["refresh_token"].as_str().unwrap().to_string();

    // Expire the rotation grace window: inside it the consumed token still
    // yields the same successor (see test_auth_refresh_grace_window).
    let _: () = redis_conn
        .del(infra::rotation_grace_key(&refresh))
        .await
        .unwrap();

    // First reuse of the old token: 401 (rotation already consumed it).
    let (resp, _) = harness.send_json_request(rotate(&refresh)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    // Second reuse = theft signal: the fresh sibling dies too.
    let (resp, _) = harness.send_json_request(rotate(&refresh)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let (resp, _) = harness.send_json_request(rotate(&fresh)).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "replay must revoke the whole family including the fresh sibling"
    );
    harness.cleanup().await;
}

/// Pair lockout after 5 failures; a different IP is unaffected.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_pair_lockout_five_failures() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let other_ip = format!("203.0.113.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let (email, password) = {
        let email = format!("lockpair_{}@example.com", Uuid::new_v4());
        harness.track_email(&email);
        let password = "LockoutPair123!";
        let signup_req = SignupRequest {
            display_name: "LockPair".to_string(),
            email: email.clone(),
            password: password.to_string(),
        };
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/signup")
            .header(common::CLIENT_IP_HEADER, &ip)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
            .unwrap();
        let (resp, body) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let registration_id = common::registration_of(&body);
        common::plant_known_otp(&mut redis_conn, &email, registration_id, common::KNOWN_OTP).await;
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify-otp")
            .header(common::CLIENT_IP_HEADER, &ip)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&VerifyOtpRequest {
                    email: email.clone(),
                    registration_id,
                    otp: "424242".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        let (resp, _) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        (email, password)
    };

    for _ in 0..5 {
        let (resp, _) = harness
            .send_json_request(login_req(&email, "WrongPassword999!", &ip))
            .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
    // 6th attempt with the CORRECT password is shed: pair locked.
    let (resp, body) = harness
        .send_json_request(login_req(&email, password, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(body["error"]["code"] == "RATE_LIMITED" || body["success"] == false);

    // Same credentials from a different IP still work (no victim lockout).
    let (resp, _) = harness
        .send_json_request(login_req(&email, password, &other_ip))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    harness.cleanup().await;
}

/// Aggregate email lockout after 20 failures across IPs.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_email_aggregate_lockout_twenty_failures() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    let base: u8 = 10 + (Uuid::new_v4().as_u128() % 150) as u8;
    let (email, password) = {
        let email = format!("lockagg_{}@example.com", Uuid::new_v4());
        harness.track_email(&email);
        let password = "LockoutAgg123!";
        let ip0 = format!("198.51.100.{base}");
        let signup_req = SignupRequest {
            display_name: "LockAgg".to_string(),
            email: email.clone(),
            password: password.to_string(),
        };
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/signup")
            .header(common::CLIENT_IP_HEADER, &ip0)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
            .unwrap();
        let (resp, body) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let registration_id = common::registration_of(&body);
        common::plant_known_otp(&mut redis_conn, &email, registration_id, common::KNOWN_OTP).await;
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify-otp")
            .header(common::CLIENT_IP_HEADER, &ip0)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&VerifyOtpRequest {
                    email: email.clone(),
                    registration_id,
                    otp: "424242".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        let (resp, _) = harness.send_json_request(req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        (email, password)
    };

    // 20 failures spread over 4 IPs (5 each — below the pair threshold of 5
    // triggering on the 5th, so use 4 per IP across 5 IPs to stay under pair).
    for i in 0..5u8 {
        let ip = format!("198.51.100.{}", base.wrapping_add(i));
        for _ in 0..4 {
            let (resp, _) = harness
                .send_json_request(login_req(&email, "WrongPassword999!", &ip))
                .await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    }
    // 21st failure from a fresh IP trips the aggregate lock.
    let fresh_ip = format!("203.0.113.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let (resp, _) = harness
        .send_json_request(login_req(&email, password, &fresh_ip))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "20 aggregate failures must lock the email"
    );
    harness.cleanup().await;
}

/// Signup SMTP-abuse cap: 11 signups/hour from one IP sheds with 429.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_signup_ip_cap_eleven_per_hour() {
    let harness = TestHarness::new().await;
    let Some(mut redis_conn) = harness.live_only().await else {
        panic!("live services required (BACA_LIVE_TEST=1 with db-up)");
    };
    // The cap counts every signup from the address for an hour, so the test
    // starts from an empty counter instead of hoping no other test drew it.
    let ip = common::unique_ip("198.18");
    let _: () = redis_conn.del(format!("otp:signup:ip:{ip}")).await.unwrap();
    let mut last_status = StatusCode::CREATED;
    for i in 0..11 {
        let signup_req = SignupRequest {
            display_name: format!("Cap{i}"),
            email: harness.track_email_owned(format!("cap{i}_{}@example.com", Uuid::new_v4())),
            password: "CapTest123!".to_string(),
        };
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/signup")
            .header(common::CLIENT_IP_HEADER, &ip)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
            .unwrap();
        // Space out to dodge the 60s per-email cooldown (distinct emails
        // anyway) — the IP cap is what we assert on the 11th.
        let (resp, _) = harness.send_json_request(req).await;
        last_status = resp.status();
        if i < 10 {
            assert_eq!(last_status, StatusCode::CREATED, "signup {i} must pass");
        }
        // Cooldown between signups avoids tripping unrelated limiters.
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }
    assert_eq!(
        last_status,
        StatusCode::TOO_MANY_REQUESTS,
        "11th signup/hour from one IP must be shed"
    );
    harness.cleanup().await;
}

/// OTP abuse: 3 wrong attempts lock the OTP for 5 minutes (even the right
/// code 429s), and an expired OTP is 400 — never a silent accept.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_otp_attempt_lock_and_expiry() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    use redis::AsyncCommands;
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let email = format!("otplock_{}@example.com", Uuid::new_v4());
    harness.track_email(&email);
    let signup_req = SignupRequest {
        display_name: "OtpLock".to_string(),
        email: email.clone(),
        password: "OtpLock123!".to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header(common::CLIENT_IP_HEADER, &ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let registration_id = common::registration_of(&body);
    common::plant_known_otp(&mut redis_conn, &email, registration_id, common::KNOWN_OTP).await;

    let verify = |otp: &str| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify-otp")
            .header(common::CLIENT_IP_HEADER, &ip)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&VerifyOtpRequest {
                    email: email.clone(),
                    registration_id,
                    otp: otp.to_string(),
                })
                .unwrap(),
            ))
            .unwrap()
    };

    for _ in 0..2 {
        let (resp, _) = harness.send_json_request(verify("000000")).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    // The third miss locks the code: 429 with Retry-After, like the login
    // lockout, so a client can tell "typo" from "locked".
    let (resp, _) = harness.send_json_request(verify("000000")).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    // The correct code inside the lock window is still rejected.
    let (resp, body) = harness.send_json_request(verify("424242")).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "RATE_LIMITED");
    let retry_after: u64 = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .expect("locked OTP must carry Retry-After");
    assert!(
        (1..=300).contains(&retry_after),
        "retry window is at most five minutes"
    );

    // Expired OTP (TTL 1s, then wait) is 400, not 500/accept.
    let email2 = format!("otpexp_{}@example.com", Uuid::new_v4());
    harness.track_email(&email2);
    let (expired, code) = infra::start_registration(&mut redis_conn, &email2, "OtpExp", "hash")
        .await
        .unwrap();
    let _: () = redis_conn
        .expire(infra::registration_key(&email2, expired), 1)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header(common::CLIENT_IP_HEADER, &ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&VerifyOtpRequest {
                email: email2,
                registration_id: expired,
                otp: code,
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    harness.cleanup().await;
}

/// Password change with default flags kills the other session for real
/// (end-to-end over HTTP, not just unit-level revocation).
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_password_change_kills_other_session_e2e() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let email = format!("pwd_{}@example.com", Uuid::new_v4());
    harness.track_email(&email);
    let password = "SessionKill123!";
    let signup_req = SignupRequest {
        display_name: "SessKiller".to_string(),
        email: email.clone(),
        password: password.to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header(common::CLIENT_IP_HEADER, &ip)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signup_req).unwrap()))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let registration_id = common::registration_of(&body);
    common::plant_known_otp(&mut redis_conn, &email, registration_id, common::KNOWN_OTP).await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/verify-otp")
        .header(common::CLIENT_IP_HEADER, &ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&VerifyOtpRequest {
                email: email.clone(),
                registration_id,
                otp: "424242".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, _) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Two sessions: A and B.
    let login = || harness.send_json_request(login_req(&email, password, &ip));
    let (_, body_a) = {
        let (resp, body) = login().await;
        assert_eq!(resp.status(), StatusCode::OK);
        (resp, body)
    };
    // Revocation compares whole-second `iat <= revoked_before`, so B must be
    // minted in a strictly later second than any possible revoke-stamp: wait
    // for the next wall-clock second boundary, then log B in.
    wait_next_second().await;
    let (resp, body_b) = login().await;
    assert_eq!(resp.status(), StatusCode::OK);
    let token_a: String = body_a["data"]["access_token"].as_str().unwrap().to_string();
    let token_b: String = body_b["data"]["access_token"].as_str().unwrap().to_string();

    // A changes the password with default flags (revoke_other_sessions None).
    // Wait for the next second boundary first so the revoke-stamp strictly
    // postdates B's iat (same-second would kill B AND A).
    wait_next_second().await;
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/password")
        .header("authorization", format!("Bearer {token_a}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&ChangePasswordRequest {
                current_password: password.to_string(),
                new_password: "EvenNewer123!".to_string(),
                revoke_other_sessions: None,
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, put_body) = harness.send_json_request(req).await;
    if resp.status() != StatusCode::OK {
        panic!(
            "PUT /password failed: status={} body={put_body}",
            resp.status()
        );
    }
    // Revocation is a global whole-second timestamp (`iat <= revoked_before`),
    // so the requester's own token A — minted before the stamp — is ALSO dead
    // despite the flag being named `revoke_other_sessions`. Asserted as-is;
    // exempting the requester is tracked separately.
    let me = |t: &str| {
        Request::builder()
            .method("GET")
            .uri("/api/v1/me")
            .header("authorization", format!("Bearer {t}"))
            .body(Body::empty())
            .unwrap()
    };
    let (resp, _) = harness.send_json_request(me(&token_b)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let (resp, _) = harness.send_json_request(me(&token_a)).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "requester token also dies under global-timestamp revocation"
    );
    harness.cleanup().await;
}

/// Guest merge cap: 100 records OK, 101 rejected with 400.
#[tokio::test]
async fn test_guest_merge_enforces_hundred_record_cap() {
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    let harness = TestHarness::new().await;
    let user_id = harness.track_user(Uuid::new_v4());
    let email = format!("mergecap_{}@example.com", user_id);
    harness.track_email(&email);
    let user = infra::entities::users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        display_name: Set("MergeCap".to_string()),
        password_hash: Set(Some("hash".to_string())),
        role: Set("reader".to_string()),
        avatar_url: Set(None),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    if user.insert(&harness.state.db).await.is_err() {
        panic!("live DB required (db-up), no silent pass");
    }
    let book_id = harness.track_book(Uuid::new_v4());
    let chapter_id = Uuid::new_v4();
    let book = infra::entities::books::ActiveModel {
        id: Set(book_id),
        title: Set("Cap Novel".to_string()),
        author: Set("Cap".to_string()),
        language: Set("en".to_string()),
        primary_theme: Set("Fiction".to_string()),
        sub_theme: Set(None),
        description: Set(String::new()),
        cover_url: Set(String::new()),
        epub_storage_path: Set("books/cap.epub".to_string()),
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
    book.insert(&harness.state.db).await.unwrap();
    let chapter = infra::entities::chapters::ActiveModel {
        id: Set(chapter_id),
        book_id: Set(book_id),
        chapter_number: Set(1),
        title: Set("Chapter 1".to_string()),
        word_count: Set(1000),
        html_content: Set("<p>x</p>".to_string()),
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

    let rec = || GuestProgressRecord {
        book_id,
        last_chapter_id: chapter_id,
        last_anchor_cfi: "epubcfi(/6/2)".to_string(),
        completion_percentage: 10.0,
        is_finished: Some(false),
        last_read_at: Some(chrono::Utc::now()),
    };
    let merge = |n: usize| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/progress/merge")
            .header("authorization", format!("Bearer {access_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&GuestMergeRequest {
                    records: (0..n).map(|_| rec()).collect(),
                })
                .unwrap(),
            ))
            .unwrap()
    };

    let (resp, body) = harness.send_json_request(merge(100)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["data"]["merged_count"], 100);

    let (resp, body) = harness.send_json_request(merge(101)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("100"));

    // Cleanup.
    use sea_orm::ConnectionTrait;
    let _ = harness
        .state
        .db
        .execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "DELETE FROM users WHERE id = $1",
            vec![user_id.into()],
        ))
        .await;
    let _ = harness
        .state
        .db
        .execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "DELETE FROM books WHERE id = $1",
            vec![book_id.into()],
        ))
        .await;
    harness.cleanup().await;
}

/// Inside the rotation grace window a consumed token returns the same
/// successor, so a second tab or a retried request never kills the family.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_refresh_grace_window() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let (_, refresh) = provision_verified_user(&harness, &mut redis_conn, "grace", &ip).await;
    let rotate = |token: &str| refresh_req(token, &ip);

    let (resp, body) = harness.send_json_request(rotate(&refresh)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let first: String = body["data"]["refresh_token"].as_str().unwrap().to_string();

    let (resp, body) = harness.send_json_request(rotate(&refresh)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body["data"]["refresh_token"].as_str().unwrap(), first);

    let (resp, _) = harness.send_json_request(rotate(&first)).await;
    assert_eq!(resp.status(), StatusCode::OK, "family must stay alive");
    harness.cleanup().await;
}

/// Two simultaneous refreshes race on GETDEL: at least one wins, nobody is
/// treated as a thief, and the family survives.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_auth_concurrent_refresh_keeps_family_alive() {
    let harness = TestHarness::new().await;
    let mut redis_conn = match harness.live_only().await {
        Some(c) => c,
        None => panic!("live services required (BACA_LIVE_TEST=1 with db-up)"),
    };
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let (_, refresh) = provision_verified_user(&harness, &mut redis_conn, "race", &ip).await;

    let (a, b) = tokio::join!(
        harness.send_json_request(refresh_req(&refresh, &ip)),
        harness.send_json_request(refresh_req(&refresh, &ip))
    );
    let outcomes = [a, b];
    let successor = outcomes
        .iter()
        .find(|(resp, _)| resp.status() == StatusCode::OK)
        .map(|(_, body)| body["data"]["refresh_token"].as_str().unwrap().to_string())
        .expect("one presenter must win the rotation");
    for (resp, _) in &outcomes {
        assert!(
            matches!(resp.status(), StatusCode::OK | StatusCode::UNAUTHORIZED),
            "unexpected status {}",
            resp.status()
        );
    }

    let (resp, _) = harness
        .send_json_request(refresh_req(&successor, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "family must stay alive");
    harness.cleanup().await;
}

/// An unverified account must answer like a wrong password and count as a
/// failed attempt, so the activation state of an address is not probeable.
#[tokio::test]
async fn test_login_inactive_account_is_indistinguishable_from_wrong_password() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Postgres + Redis required (db-up)");
    let email = format!("dormant_{}@example.com", Uuid::new_v4());
    harness.track_email(&email);
    let password = "DormantSecret123!";
    let hash = infra::hash_password_async(password.to_string())
        .await
        .unwrap();
    infra::create_inactive_user(&harness.state.db, &email, "Dormant", &hash)
        .await
        .expect("inactive seed must succeed");
    let ip = format!("198.51.101.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);

    let (resp, body) = harness
        .send_json_request(login_req(&email, password, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["message"], "Invalid email or password");

    let failures: i64 = redis_conn
        .get(format!("auth:login:fail:{email}:{ip}"))
        .await
        .unwrap_or(0);
    assert_eq!(failures, 1, "the attempt must count toward lockout");
    harness.cleanup().await;
}

fn refresh_req(token: &str, ip: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header(common::CLIENT_IP_HEADER, ip)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&RefreshTokenRequest {
                refresh_token: token.to_string(),
            })
            .unwrap(),
        ))
        .unwrap()
}

/// Two devices sign in; the shelf lists both with the caller flagged, one
/// device can be dropped alone (its refresh token dies), and "sign out
/// everywhere but here" keeps the caller signed in.
#[tokio::test]
#[ignore = "needs live Postgres + Redis (BACA_LIVE_TEST=1)"]
async fn test_sessions_list_single_revoke_and_keep_current() {
    let harness = TestHarness::new().await;
    let mut redis_conn = harness
        .live_only()
        .await
        .expect("live Redis required (db-up)");
    let ip = format!("198.51.100.{}", 10 + (Uuid::new_v4().as_u128() % 200) as u8);
    let (phone_access, phone_refresh) =
        provision_verified_user(&harness, &mut redis_conn, "sess", &ip).await;
    let email = {
        let claims =
            infra::verify_access_token(&phone_access, harness.state.config.jwt_secret()).unwrap();
        claims.email
    };

    let laptop_login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(common::CLIENT_IP_HEADER, "203.0.113.77")
        .header(
            "user-agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0",
        )
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&LoginRequest {
                email: email.clone(),
                password: "Adversarial123!".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();
    let (resp, body) = harness.send_json_request(laptop_login).await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let laptop_access = body["data"]["access_token"].as_str().unwrap().to_string();
    let laptop_refresh = body["data"]["refresh_token"].as_str().unwrap().to_string();

    let sessions = |token: &str| {
        Request::builder()
            .method("GET")
            .uri("/api/v1/me/sessions")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    };
    let (resp, body) = harness.send_json_request(sessions(&laptop_access)).await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let list = body["data"].as_array().expect("sessions array");
    assert_eq!(list.len(), 2, "{body}");
    let current: Vec<&Value> = list.iter().filter(|s| s["current"] == true).collect();
    assert_eq!(current.len(), 1, "exactly one current session");
    assert_eq!(current[0]["device"], "Firefox \u{00B7} Linux");
    assert_eq!(current[0]["ip_prefix"], "203.0.113.x");
    let phone_id = list
        .iter()
        .find(|s| s["current"] == false)
        .and_then(|s| s["id"].as_str())
        .expect("other session id")
        .to_string();

    let revoke = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/me/sessions/{phone_id}"))
        .header("authorization", format!("Bearer {laptop_access}"))
        .body(Body::empty())
        .unwrap();
    let (resp, _) = harness.send_json_request(revoke).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let (resp, _) = harness
        .send_json_request(refresh_req(&phone_refresh, &ip))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "the dropped device cannot refresh"
    );
    let (resp, _) = harness.send_json_request(sessions(&phone_access)).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "its last access token is blacklisted"
    );
    let (_, body) = harness.send_json_request(sessions(&laptop_access)).await;
    assert_eq!(body["data"].as_array().map(Vec::len), Some(1));

    let revoke_again = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/me/sessions/{phone_id}"))
        .header("authorization", format!("Bearer {laptop_access}"))
        .body(Body::empty())
        .unwrap();
    let (resp, _) = harness.send_json_request(revoke_again).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let (resp, body) = harness
        .send_json_request(refresh_req(&laptop_refresh, "203.0.113.77"))
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let laptop_access = body["data"]["access_token"].as_str().unwrap().to_string();
    let keep_current = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/revoke-all?keep_current=true")
        .header("authorization", format!("Bearer {laptop_access}"))
        .body(Body::empty())
        .unwrap();
    let (resp, body) = harness.send_json_request(keep_current).await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let (resp, body) = harness.send_json_request(sessions(&laptop_access)).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "the caller survives keep_current: {body}"
    );
    assert_eq!(body["data"].as_array().map(Vec::len), Some(1));
    harness.cleanup().await;
}

fn json_post(uri: &str, ip: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(common::CLIENT_IP_HEADER, ip)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn signup_body(email: &str, password: &str) -> Value {
    serde_json::json!({ "display_name": "Pending Owner", "email": email, "password": password })
}

/// Signs `email` up with `password` and pins the code of that registration
/// to `otp`; returns the registration id.
async fn signup_with_code(
    harness: &TestHarness,
    redis: &mut infra::RedisConn,
    ip: &str,
    email: &str,
    password: &str,
    otp: &str,
) -> Uuid {
    let (resp, body) = harness
        .send_json_request(json_post(
            "/api/v1/auth/signup",
            ip,
            signup_body(email, password),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{body}");
    let registration_id = common::registration_of(&body);
    common::plant_known_otp(redis, email, registration_id, otp).await;
    registration_id
}

fn verify_req(email: &str, registration_id: Uuid, otp: &str, ip: &str) -> Request<Body> {
    json_post(
        "/api/v1/auth/verify-otp",
        ip,
        serde_json::json!({ "email": email, "registration_id": registration_id, "otp": otp }),
    )
}

/// Someone else signing up the owner's address first holds nothing: both
/// registrations live, and a code only activates the password of the
/// registration it was mailed for.
#[tokio::test]
async fn test_registrations_for_one_address_live_side_by_side() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let email = harness.track_email_owned(format!("pending_{}@example.com", Uuid::new_v4()));
    let owner_pw = "OwnerChoice123!";
    let squatter_pw = "AttackerPick123!";

    let squatter = signup_with_code(&harness, &mut redis, &ip, &email, squatter_pw, "111111").await;
    let owner = signup_with_code(&harness, &mut redis, &ip, &email, owner_pw, "424242").await;
    let unverified = infra::find_user_by_email(&harness.state.db, &email)
        .await
        .unwrap();
    assert!(unverified.is_none(), "a signup writes no account row");

    let resp = harness
        .send_request(verify_req(&email, owner, "111111", &ip))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "the squatter's code does not open the owner's registration"
    );
    let resp = harness
        .send_request(verify_req(&email, owner, "424242", &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = harness.send_request(login_req(&email, owner_pw, &ip)).await;
    assert_eq!(resp.status(), StatusCode::OK, "owner password is the one");
    let resp = harness
        .send_request(login_req(&email, squatter_pw, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let resp = harness
        .send_request(verify_req(&email, squatter, "111111", &ip))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a late registration cannot take over an active account"
    );
    harness.cleanup().await;
}

/// Codes mailed to one address are capped at three per hour and ten per
/// day; the refused signup sends nothing and says when to come back.
#[tokio::test]
async fn test_code_mail_is_capped_per_recipient() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let email = harness.track_email_owned(format!("bomb_{}@example.com", Uuid::new_v4()));
    for _ in 0..3 {
        signup_with_code(&harness, &mut redis, &ip, &email, "BombOwner123!", "424242").await;
    }
    let (resp, body) = harness
        .send_json_request(json_post(
            "/api/v1/auth/signup",
            &ip,
            signup_body(&email, "BombOwner123!"),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS, "{body}");
    let retry_after: u64 = resp.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        (1..=3600).contains(&retry_after),
        "retry after {retry_after}"
    );
    let pending: Vec<String> = redis.keys(format!("otp:{email}:*")).await.unwrap();
    assert_eq!(pending.len(), 3, "the refused signup stored no code");

    let other = harness.track_email_owned(format!("bombday_{}@example.com", Uuid::new_v4()));
    let _: () = redis
        .set_ex(format!("otp:sent:86400:{other}"), 10, 3600)
        .await
        .unwrap();
    let resp = harness
        .send_request(json_post(
            "/api/v1/auth/signup",
            &ip,
            signup_body(&other, "BombOwner123!"),
        ))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "the eleventh code of the day is refused"
    );
    harness.cleanup().await;
}

/// Guesses are counted per address over a day: new codes do not refill
/// them, and once they are spent no new code is issued.
#[tokio::test]
async fn test_spent_guesses_stop_new_codes() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let email = harness.track_email_owned(format!("guess_{}@example.com", Uuid::new_v4()));
    let first = signup_with_code(
        &harness,
        &mut redis,
        &ip,
        &email,
        "GuessOwner123!",
        "424242",
    )
    .await;
    for _ in 0..2 {
        let resp = harness
            .send_request(verify_req(&email, first, "000000", &ip))
            .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    let _: () = redis
        .set_ex(format!("otp:guesses:{email}"), 10, 86_400)
        .await
        .unwrap();
    let second = signup_with_code_refused(&harness, &ip, &email).await;
    assert!(
        (1..=86_400).contains(&second),
        "no new code until the day is over, retry after {second}"
    );
    let resp = harness
        .send_request(verify_req(&email, first, "424242", &ip))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "even the right code waits once the guesses are spent"
    );
    harness.cleanup().await;
}

/// A signup refused with 429; returns its Retry-After.
async fn signup_with_code_refused(harness: &TestHarness, ip: &str, email: &str) -> u64 {
    let resp = harness
        .send_request(json_post(
            "/api/v1/auth/signup",
            ip,
            signup_body(email, "GuessOwner123!"),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    resp.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap()
}

/// Ten wrong guesses fired at once: the count is atomic, so at most three
/// are compared and the rest answer 429, and the right code is then locked.
#[tokio::test]
async fn test_parallel_wrong_otps_are_counted_atomically() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let email = harness.track_email_owned(format!("otprace_{}@example.com", Uuid::new_v4()));
    let registration =
        signup_with_code(&harness, &mut redis, &ip, &email, "RaceOwner123!", "424242").await;

    let statuses: Vec<StatusCode> = futures::future::join_all(
        (0..10).map(|_| harness.send_request(verify_req(&email, registration, "000000", &ip))),
    )
    .await
    .iter()
    .map(|r| r.status())
    .collect();
    let compared = statuses
        .iter()
        .filter(|s| **s == StatusCode::BAD_REQUEST)
        .count();
    let refused = statuses
        .iter()
        .filter(|s| **s == StatusCode::TOO_MANY_REQUESTS)
        .count();
    assert!(compared <= 3, "{statuses:?}");
    assert_eq!(compared + refused, 10, "{statuses:?}");

    let resp = harness
        .send_request(verify_req(&email, registration, "424242", &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    harness.cleanup().await;
}

/// The grace window hands back a successor only while it is alive:
/// after revoke-all, replaying the consumed token is a 401.
#[tokio::test]
async fn test_refresh_grace_after_revoke_all_is_refused() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (_, first_refresh) = provision_verified_user(&harness, &mut redis, "grace", &ip).await;

    let (resp, rotated) = harness
        .send_json_request(refresh_req(&first_refresh, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let access = rotated["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let revoke = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/revoke-all")
        .header(common::CLIENT_IP_HEADER, &ip)
        .header("authorization", format!("Bearer {access}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(harness.send_request(revoke).await.status(), StatusCode::OK);

    let resp = harness.send_request(refresh_req(&first_refresh, &ip)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    harness.cleanup().await;
}

/// When the revocation stamp cannot be written, the password change fails
/// before the hash is replaced: the old password still signs in.
#[tokio::test]
async fn test_password_change_keeps_old_hash_when_revocation_fails() {
    let base = infra::AppConfig::from_env().unwrap_or_default();
    let relay = common::TcpRelay::start_tripping(
        &common::redis_upstream(&base),
        &["SETEX", "user_revoked_before:"],
    )
    .await;
    let relay_url = common::redis_url_via(&base, relay.port());
    let harness = TestHarness::with_config(|c| c.redis.url = relay_url).await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (access, _) = provision_verified_user(&harness, &mut redis, "pwfail", &ip).await;
    let me = Request::builder()
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access}"))
        .body(Body::empty())
        .unwrap();
    let (_, profile) = harness.send_json_request(me).await;
    let email = profile["data"]["email"].as_str().unwrap().to_string();

    let change = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/password")
        .header("authorization", format!("Bearer {access}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "current_password": common::PROVISIONED_PASSWORD,
                "new_password": "Replacement123!"
            })
            .to_string(),
        ))
        .unwrap();
    let resp = harness.send_request(change).await;
    assert!(resp.status().is_server_error(), "got {}", resp.status());

    let resp = harness
        .send_request(login_req(&email, common::PROVISIONED_PASSWORD, &ip))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "old password must still work"
    );
    let resp = harness
        .send_request(login_req(&email, "Replacement123!", &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    harness.cleanup().await;
}

/// Case and padding variants of one address are one account and one
/// lockout key; a second signup for it answers like any signup.
#[tokio::test]
async fn test_email_variants_are_one_account() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let id = Uuid::new_v4();
    let canonical = harness.track_email_owned(format!("case_{id}@example.com"));
    let shouted = format!("  Case_{id}@Example.COM ");
    let password = "CaseFolded123!";

    let (resp, body) = harness
        .send_json_request(json_post(
            "/api/v1/auth/signup",
            &ip,
            signup_body(&shouted, password),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let registration = common::registration_of(&body);
    common::plant_known_otp(&mut redis, &canonical, registration, "424242").await;
    let resp = harness
        .send_request(verify_req(&shouted, registration, "424242", &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);

    for variant in [canonical.as_str(), shouted.as_str()] {
        let resp = harness
            .send_request(login_req(variant, password, &ip))
            .await;
        assert_eq!(resp.status(), StatusCode::OK, "login as {variant:?}");
    }
    let resp = harness
        .send_request(json_post(
            "/api/v1/auth/signup",
            &ip,
            signup_body(&canonical.to_uppercase(), password),
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    harness.cleanup().await;
}

/// Signup does not tell who is registered: an address that has an account
/// answers with the same status and body shape as a new one, and its owner
/// is told by mail instead of being sent a code.
#[tokio::test]
async fn test_signup_answers_alike_for_registered_and_new_addresses() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (access, _) = provision_verified_user(&harness, &mut redis, "known", &ip).await;
    let registered = infra::verify_access_token(&access, harness.state.config.jwt_secret())
        .unwrap()
        .email;
    let fresh = harness.track_email_owned(format!("fresh_{}@example.com", Uuid::new_v4()));

    let mut answers = Vec::new();
    for email in [&registered, &fresh] {
        let (resp, body) = harness
            .send_json_request(json_post(
                "/api/v1/auth/signup",
                &ip,
                signup_body(email, "AnyChoice123!"),
            ))
            .await;
        assert_eq!(resp.status(), StatusCode::CREATED, "{body}");
        let data = body["data"].as_object().expect("data").clone();
        answers.push((
            data.keys().cloned().collect::<Vec<_>>(),
            data["message"].clone(),
        ));
    }
    assert_eq!(answers[0], answers[1]);
    harness.cleanup().await;
}

/// Two registrations of one address verified at once never answer 5xx:
/// one account is created and the loser of the race is a conflict.
#[tokio::test]
async fn test_concurrent_verifications_never_fail_with_5xx() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let email = harness.track_email_owned(format!("twin_{}@example.com", Uuid::new_v4()));
    let a = signup_with_code(
        &harness,
        &mut redis,
        &ip,
        &email,
        "TwinSignup123!",
        "424242",
    )
    .await;
    let b = signup_with_code(
        &harness,
        &mut redis,
        &ip,
        &email,
        "TwinSignup123!",
        "424242",
    )
    .await;
    let (first, second) = tokio::join!(
        harness.send_request(verify_req(&email, a, "424242", &ip)),
        harness.send_request(verify_req(&email, b, "424242", &ip)),
    );
    let mut statuses = [first.status(), second.status()];
    statuses.sort();
    assert_eq!(
        statuses,
        [StatusCode::OK, StatusCode::CONFLICT],
        "{statuses:?}"
    );

    let other = harness.track_email_owned(format!("twice_{}@example.com", Uuid::new_v4()));
    let once = signup_with_code(
        &harness,
        &mut redis,
        &ip,
        &other,
        "TwinSignup123!",
        "424242",
    )
    .await;
    let (first, second) = tokio::join!(
        harness.send_request(verify_req(&other, once, "424242", &ip)),
        harness.send_request(verify_req(&other, once, "424242", &ip)),
    );
    let mut statuses = [first.status(), second.status()];
    statuses.sort();
    assert_eq!(
        statuses,
        [StatusCode::OK, StatusCode::BAD_REQUEST],
        "one registration is consumed once: {statuses:?}"
    );
    harness.cleanup().await;
}

/// Guesses at a locked code are refused without touching the address's
/// daily budget, so one registration cannot be used to spend it.
#[tokio::test]
async fn test_guesses_at_a_locked_code_spend_no_daily_budget() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let email = harness.track_email_owned(format!("lockburn_{}@example.com", Uuid::new_v4()));
    let squatter =
        signup_with_code(&harness, &mut redis, &ip, &email, "Squatter123!", "424242").await;
    for _ in 0..12 {
        harness
            .send_request(verify_req(&email, squatter, "000000", &ip))
            .await;
    }
    let spent: i64 = redis.get(format!("otp:guesses:{email}")).await.unwrap();
    assert_eq!(spent, 3, "only the three compared guesses count");
    signup_with_code(&harness, &mut redis, &ip, &email, "RealOwner123!", "135791").await;
    harness.cleanup().await;
}

/// A refresh token that outlived a device sign-out (a rotation racing the
/// sign-out) is refused: the session revocation covers refreshes too.
#[tokio::test]
async fn test_refresh_of_a_revoked_session_is_refused() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (access, refresh) = provision_verified_user(&harness, &mut redis, "zombie", &ip).await;
    let sid = infra::verify_access_token(&access, harness.state.config.jwt_secret())
        .unwrap()
        .sid
        .expect("session id");
    infra::revoke_session_tokens(&mut redis, sid, 3600)
        .await
        .unwrap();
    let (resp, body) = harness.send_json_request(refresh_req(&refresh, &ip)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "{body}");
    harness.cleanup().await;
}

/// Two tabs refreshing with the same token at once both get access tokens
/// of the same session, so signing the device out reaches both.
#[tokio::test]
async fn test_concurrent_refreshes_stay_in_one_session() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (access, refresh) = provision_verified_user(&harness, &mut redis, "twotabs", &ip).await;
    let sid = infra::verify_access_token(&access, harness.state.config.jwt_secret())
        .unwrap()
        .sid;
    let (a, b) = tokio::join!(
        harness.send_json_request(refresh_req(&refresh, &ip)),
        harness.send_json_request(refresh_req(&refresh, &ip)),
    );
    for (resp, body) in [a, b] {
        assert_eq!(resp.status(), StatusCode::OK, "{body}");
        let token = body["data"]["access_token"].as_str().unwrap();
        let claims = infra::verify_access_token(token, harness.state.config.jwt_secret()).unwrap();
        assert_eq!(claims.sid, sid, "both tabs stay in the session");
    }
    harness.cleanup().await;
}

fn me_req(access: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {access}"))
        .body(Body::empty())
        .unwrap()
}

/// Signing one device out kills every access token that device was ever
/// given, not only the newest: refreshes keep the session id.
#[tokio::test]
async fn test_device_sign_out_revokes_every_access_token_of_the_session() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (first_access, refresh) = provision_verified_user(&harness, &mut redis, "sid", &ip).await;
    let email = infra::verify_access_token(&first_access, harness.state.config.jwt_secret())
        .unwrap()
        .email;

    let mut accesses = vec![first_access];
    let mut refresh = refresh;
    for _ in 0..2 {
        let (resp, body) = harness.send_json_request(refresh_req(&refresh, &ip)).await;
        assert_eq!(resp.status(), StatusCode::OK, "{body}");
        accesses.push(body["data"]["access_token"].as_str().unwrap().to_string());
        refresh = body["data"]["refresh_token"].as_str().unwrap().to_string();
    }
    let sids: Vec<_> = accesses
        .iter()
        .map(|a| {
            infra::verify_access_token(a, harness.state.config.jwt_secret())
                .unwrap()
                .sid
        })
        .collect();
    assert!(
        sids[0].is_some() && sids.iter().all(|s| *s == sids[0]),
        "{sids:?}"
    );

    let (resp, body) = harness
        .send_json_request(login_req(&email, common::PROVISIONED_PASSWORD, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let laptop = body["data"]["access_token"].as_str().unwrap().to_string();
    let (_, body) = harness
        .send_json_request(
            Request::builder()
                .method("GET")
                .uri("/api/v1/me/sessions")
                .header("authorization", format!("Bearer {laptop}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let phone = body["data"]
        .as_array()
        .and_then(|list| list.iter().find(|s| s["current"] == false))
        .and_then(|s| s["id"].as_str())
        .expect("the phone session")
        .to_string();
    let resp = harness
        .send_request(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/me/sessions/{phone}"))
                .header("authorization", format!("Bearer {laptop}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK);

    for (i, access) in accesses.iter().enumerate() {
        let resp = harness.send_request(me_req(access)).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "access token {i}");
    }
    let resp = harness.send_request(me_req(&laptop)).await;
    assert_eq!(resp.status(), StatusCode::OK, "the other device stays in");
    harness.cleanup().await;
}

/// Twenty wrong passwords for one address from one network at once: the
/// attempt is counted before Argon2, so exactly five are evaluated.
#[tokio::test]
async fn test_parallel_login_guesses_evaluate_only_five() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (access, _) = provision_verified_user(&harness, &mut redis, "burst", &ip).await;
    let email = infra::verify_access_token(&access, harness.state.config.jwt_secret())
        .unwrap()
        .email;
    let guesser = common::unique_ip("198.19");
    let statuses: Vec<StatusCode> = futures::future::join_all(
        (0..20).map(|_| harness.send_request(login_req(&email, "WrongGuess123!", &guesser))),
    )
    .await
    .iter()
    .map(|r| r.status())
    .collect();
    let evaluated = statuses
        .iter()
        .filter(|s| **s == StatusCode::UNAUTHORIZED)
        .count();
    let refused = statuses
        .iter()
        .filter(|s| **s == StatusCode::TOO_MANY_REQUESTS)
        .count();
    assert_eq!((evaluated, refused), (5, 15), "{statuses:?}");
    harness.cleanup().await;
}

/// A pending (never verified) account and an unknown address answer a
/// login in the same time: both run Argon2 before anything branches.
/// Timing needs a quiet machine, so it runs single-threaded with the live
/// suite.
#[tokio::test]
#[ignore = "timing; runs single-threaded under make test-live"]
async fn test_login_time_does_not_tell_pending_from_unknown() {
    let harness = TestHarness::new().await;
    harness.live_only().await.expect("live Redis required");
    let pending = harness.track_email_owned(format!("pendtime_{}@example.com", Uuid::new_v4()));
    let hash = infra::hash_password_async("PendingTime123!".to_string())
        .await
        .unwrap();
    infra::create_inactive_user(&harness.state.db, &pending, "Pending", &hash)
        .await
        .unwrap();
    let unknown = format!("nobody_{}@example.com", Uuid::new_v4());

    // Interleaved, after a warm-up, so both series see the same load.
    let shared = &harness;
    let timed = |email: String| async move {
        let harness = shared;
        let started = std::time::Instant::now();
        let resp = harness
            .send_request(login_req(
                &email,
                "PendingTime123!",
                &common::unique_ip("198.19"),
            ))
            .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        started.elapsed().as_secs_f64() * 1000.0
    };
    timed(format!("warmup_{}@example.com", Uuid::new_v4())).await;
    let (mut pending_times, mut unknown_times) = (Vec::new(), Vec::new());
    for _ in 0..20 {
        pending_times.push(timed(pending.clone()).await);
        unknown_times.push(timed(unknown.clone()).await);
    }
    let median = |mut times: Vec<f64>| {
        times.sort_by(f64::total_cmp);
        times[times.len() / 2]
    };
    let (pending_ms, unknown_ms) = (median(pending_times), median(unknown_times));
    assert!(
        (pending_ms - unknown_ms).abs() < 10.0,
        "pending {pending_ms:.1} ms, unknown {unknown_ms:.1} ms"
    );
    harness.cleanup().await;
}

/// A wrong current password is the form's problem, not the session's: 403,
/// counted once toward the change lockout, and the session keeps working.
#[tokio::test]
async fn test_wrong_current_password_is_403_and_keeps_the_session() {
    let harness = TestHarness::new().await;
    let mut redis = harness.live_only().await.expect("live Redis required");
    let ip = common::unique_ip("198.19");
    let (access, _) = provision_verified_user(&harness, &mut redis, "pwdwrong", &ip).await;
    let user_id = infra::verify_access_token(&access, harness.state.config.jwt_secret())
        .unwrap()
        .sub;
    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/password")
        .header("authorization", format!("Bearer {access}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "current_password": "NotMyPassword123!", "new_password": "BrandNewPass123!" })
                .to_string(),
        ))
        .unwrap();
    let (resp, body) = harness.send_json_request(req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    let counted: i64 = redis
        .get(format!("auth:pwd_change:fail:{user_id}"))
        .await
        .unwrap();
    assert_eq!(counted, 1);
    let resp = harness.send_request(me_req(&access)).await;
    assert_eq!(resp.status(), StatusCode::OK, "the session is untouched");
    harness.cleanup().await;
}
