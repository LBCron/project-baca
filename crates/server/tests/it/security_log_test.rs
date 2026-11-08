//! Security events and request lines: each event once, with stable names,
//! and never a token, code, password, or full email address.

use crate::common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{LogCapture, TestHarness, CLIENT_IP_HEADER, KNOWN_OTP, PROVISIONED_PASSWORD};
use pretty_assertions::assert_eq;
use redis::AsyncCommands;
use serde_json::{json, Value};
use std::time::Duration;
use uuid::Uuid;

const WRONG_PASSWORD: &str = "Wrong-Guess-77!";
const WRONG_OTP: &str = "000000";

fn post(uri: &str, ip: &str, body: Value, bearer: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(CLIENT_IP_HEADER, ip)
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn with_bearer(method: &str, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn signup(email: &str, ip: &str) -> Request<Body> {
    post(
        "/api/v1/auth/signup",
        ip,
        json!({ "display_name": "Log Probe", "email": email, "password": PROVISIONED_PASSWORD }),
        None,
    )
}

fn verify(email: &str, registration_id: Uuid, otp: &str, ip: &str) -> Request<Body> {
    post(
        "/api/v1/auth/verify-otp",
        ip,
        json!({ "email": email, "registration_id": registration_id, "otp": otp }),
        None,
    )
}

fn login(email: &str, password: &str, ip: &str) -> Request<Body> {
    post(
        "/api/v1/auth/login",
        ip,
        json!({ "email": email, "password": password }),
        None,
    )
}

fn refresh(token: &str, ip: &str) -> Request<Body> {
    post(
        "/api/v1/auth/refresh",
        ip,
        json!({ "refresh_token": token }),
        None,
    )
}

fn data_str(body: &Value, field: &str) -> String {
    body["data"][field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} missing: {body}"))
        .to_string()
}

/// One account from signup to deletion: every security event appears once,
/// a replayed refresh token raises exactly one warning naming the account,
/// and no secret reaches the log at any level.
#[tokio::test]
async fn test_auth_journey_logs_each_event_once_and_no_secret() {
    let harness = TestHarness::new().await;
    let mut redis = harness
        .live_only()
        .await
        .expect("live Postgres + Redis required (db-up)");
    let ip = common::unique_ip("198.23");
    let email = harness.track_email_owned(format!("log_{}@example.com", Uuid::new_v4()));
    let logs = LogCapture::start();
    let mut requests = 0;
    let mut secrets = vec![
        email.clone(),
        PROVISIONED_PASSWORD.to_string(),
        WRONG_PASSWORD.to_string(),
        KNOWN_OTP.to_string(),
        WRONG_OTP.to_string(),
    ];

    let (resp, body) = harness.send_json_request(signup(&email, &ip)).await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::CREATED, "{body}");
    let registration_id = common::registration_of(&body);
    common::plant_known_otp(&mut redis, &email, registration_id, KNOWN_OTP).await;

    let resp = harness
        .send_request(verify(&email, registration_id, WRONG_OTP, &ip))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let (resp, body) = harness
        .send_json_request(verify(&email, registration_id, KNOWN_OTP, &ip))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let user_id = body["data"]["user"]["id"].as_str().unwrap().to_string();
    let first_refresh = data_str(&body, "refresh_token");
    secrets.push(data_str(&body, "access_token"));
    secrets.push(first_refresh.clone());

    // A second signup for the now registered address.
    let (resp, _) = harness.send_json_request(signup(&email, &ip)).await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = harness
        .send_request(login(&email, WRONG_PASSWORD, &ip))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let (resp, body) = harness
        .send_json_request(refresh(&first_refresh, &ip))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let rotated_access = data_str(&body, "access_token");
    let rotated_refresh = data_str(&body, "refresh_token");
    secrets.push(rotated_access.clone());
    secrets.push(rotated_refresh.clone());

    let (resp, _) = harness
        .send_json_request(post(
            "/api/v1/auth/logout",
            &ip,
            json!({ "refresh_token": rotated_refresh }),
            Some(&rotated_access),
        ))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::OK);

    // Past the grace window a rotated-out token is theft, not a second tab.
    let _: () = redis
        .del(infra::rotation_grace_key(&first_refresh))
        .await
        .unwrap();
    let before_replay = logs.text().len();
    let resp = harness.send_request(refresh(&first_refresh, &ip)).await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let warnings: Vec<Value> = logs.text()[before_replay..]
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|line| line["level"] == "WARN")
        .collect();
    assert_eq!(warnings.len(), 1, "replay warnings: {warnings:?}");
    assert_eq!(
        warnings[0]["fields"]["event"],
        "auth.refresh.replay_detected"
    );
    assert_eq!(warnings[0]["fields"]["user_id"], user_id);

    // The replay cut every token issued up to this second.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let (resp, body) = harness
        .send_json_request(login(&email, PROVISIONED_PASSWORD, &ip))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::OK, "{body}");
    let access = data_str(&body, "access_token");
    secrets.push(access.clone());
    secrets.push(data_str(&body, "refresh_token"));

    let resp = harness
        .send_request(with_bearer("GET", "/api/v1/me?probe=query-secret", &access))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::OK);
    secrets.push("query-secret".to_string());

    let resp = harness
        .send_request(with_bearer("DELETE", "/api/v1/me", &access))
        .await;
    requests += 1;
    assert_eq!(resp.status(), StatusCode::OK);

    for event in [
        "auth.signup.started",
        "auth.otp.sent",
        "auth.otp.failed",
        "auth.signup.verified",
        "auth.signup.existing_account",
        "auth.login.failed",
        "auth.login.succeeded",
        "auth.session.revoked",
        "auth.refresh.replay_detected",
        "auth.account.deleted",
    ] {
        logs.only(event);
    }
    assert_eq!(
        logs.only("auth.signup.verified")["fields"]["user_id"],
        user_id
    );
    assert_eq!(
        logs.only("auth.account.deleted")["fields"]["user_id"],
        user_id
    );
    assert_eq!(
        logs.only("auth.session.revoked")["fields"]["scope"],
        "logout"
    );

    let failed = logs.only("auth.login.failed");
    assert_eq!(failed["level"], "WARN");
    assert_eq!(failed["fields"]["reason"], "wrong_password");
    assert_eq!(failed["fields"]["user_id"], user_id);
    assert_eq!(failed["fields"]["email_hash"].as_str().unwrap().len(), 16);
    assert!(
        failed["fields"]["client_prefix"]
            .as_str()
            .unwrap()
            .ends_with(".0/24"),
        "{failed}"
    );
    assert!(failed["span"]["request_id"].is_string(), "{failed}");
    assert_eq!(
        failed["fields"]["email_hash"],
        logs.only("auth.signup.started")["fields"]["email_hash"],
        "one address hashes the same everywhere"
    );

    let completed = logs.events("http.request.completed");
    assert_eq!(completed.len(), requests, "one completion line per request");
    for line in &completed {
        let fields = &line["fields"];
        assert!(
            fields["method"].is_string() && fields["status"].is_u64(),
            "{line}"
        );
        assert!(
            fields["latency_ms"].is_u64() && fields["request_id"].is_string(),
            "{line}"
        );
        assert!(
            fields["route"].as_str().unwrap().starts_with("/api/v1/"),
            "{line}"
        );
    }
    assert!(completed
        .iter()
        .any(|line| line["fields"]["route"] == "/api/v1/me" && line["fields"]["method"] == "GET"));

    let text = logs.text();
    for secret in &secrets {
        assert!(!text.contains(secret.as_str()), "log holds {secret}");
    }
    assert!(!text.contains("birth"), "no age field is ever logged");
    harness.cleanup().await;
}

/// Five wrong passwords lock the address for this network once; the next
/// attempt is refused as locked. Three wrong codes lock a registration.
#[tokio::test]
async fn test_lockouts_are_logged_once() {
    let harness = TestHarness::new().await;
    harness
        .live_only()
        .await
        .expect("live Postgres + Redis required (db-up)");
    let ip = common::unique_ip("198.23");
    let email = format!("nobody_{}@example.com", Uuid::new_v4());
    let logs = LogCapture::start();

    for _ in 0..5 {
        let resp = harness
            .send_request(login(&email, WRONG_PASSWORD, &ip))
            .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
    let resp = harness
        .send_request(login(&email, WRONG_PASSWORD, &ip))
        .await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    let lock = logs.only("auth.lockout.set");
    assert_eq!(lock["fields"]["scope"], "address_and_network");
    let failures = logs.events("auth.login.failed");
    assert_eq!(failures.len(), 6);
    assert!(failures[..5]
        .iter()
        .all(|f| f["fields"]["reason"] == "unknown_account" && f["fields"]["user_id"].is_null()));
    assert_eq!(failures[5]["fields"]["reason"], "locked");

    let signup_email = harness.track_email_owned(format!("otp_{}@example.com", Uuid::new_v4()));
    let (resp, body) = harness.send_json_request(signup(&signup_email, &ip)).await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{body}");
    let registration_id = common::registration_of(&body);
    for expected in [
        StatusCode::BAD_REQUEST,
        StatusCode::BAD_REQUEST,
        StatusCode::TOO_MANY_REQUESTS,
    ] {
        let resp = harness
            .send_request(verify(&signup_email, registration_id, WRONG_OTP, &ip))
            .await;
        assert_eq!(resp.status(), expected);
    }
    assert_eq!(logs.events("auth.otp.failed").len(), 2);
    logs.only("auth.otp.locked");

    let text = logs.text();
    for secret in [&email, &signup_email, WRONG_PASSWORD, WRONG_OTP] {
        assert!(!text.contains(secret), "log holds {secret}");
    }
    harness.cleanup().await;
}

/// SMTP relay that rejects every recipient the way real relays do, quoting
/// the address in its reply.
async fn rejecting_relay() -> u16 {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let (read, mut write) = socket.into_split();
                let mut lines = BufReader::new(read).lines();
                let _ = write.write_all(b"220 relay.test ESMTP\r\n").await;
                while let Ok(Some(line)) = lines.next_line().await {
                    let command = line.to_ascii_uppercase();
                    let reply = match line.get(..8) {
                        Some(verb) if verb.eq_ignore_ascii_case("RCPT TO:") => format!(
                            "550 5.1.1 {}: Recipient address rejected\r\n",
                            line[8..].trim()
                        ),
                        _ if command.starts_with("QUIT") => "221 bye\r\n".to_string(),
                        _ => "250 OK\r\n".to_string(),
                    };
                    if write.write_all(reply.as_bytes()).await.is_err()
                        || command.starts_with("QUIT")
                    {
                        break;
                    }
                }
            });
        }
    });
    port
}

/// A relay that rejects the recipient, and one that is not there at all:
/// signup answers 502 and no log line names the address, masked or not,
/// nor repeats the relay's reply text.
#[tokio::test]
async fn test_smtp_failures_never_log_the_address() {
    let rejecting = rejecting_relay().await;
    let closed = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    for (port, kind) in [
        (rejecting, "permanent rejection 550"),
        (closed, "connection"),
    ] {
        let harness = TestHarness::with_config(|c| {
            c.email.smtp_host = "127.0.0.1".to_string();
            c.email.smtp_port = port;
            c.email.smtp_security = infra::SmtpSecurity::None;
            c.email.smtp_user = String::new();
        })
        .await;
        harness
            .live_only()
            .await
            .expect("live Postgres + Redis required (db-up)");
        let local = format!("smtp_{}", Uuid::new_v4().simple());
        let email = harness.track_email_owned(format!("{local}@rejected.example"));
        let logs = LogCapture::start();

        let (resp, body) = harness
            .send_json_request(signup(&email, &common::unique_ip("198.23")))
            .await;
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY, "{body}");

        let text = logs.text();
        assert!(text.contains(kind), "failure kind {kind} logged: {text}");
        for leak in [
            "@",
            "***",
            "rejected.example",
            local.as_str(),
            "Recipient address",
        ] {
            assert!(!text.contains(leak), "log holds {leak}: {text}");
        }
        drop(logs);
        harness.cleanup().await;
    }
}
