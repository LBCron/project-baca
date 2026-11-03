//! Auth and user endpoints.

use crate::{
    error::HttpError,
    middleware::{
        auth::AuthUser,
        rate_limit::{client_ip_from_headers, client_subject_from_headers},
    },
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use chrono::Utc;
use infra::{
    activate_registration, blacklist_access_token, claim_code_quota, delete_user_by_id,
    ensure_guesses_left, entities::users, find_user_by_email, find_user_by_id,
    generate_refresh_token, hit_window, is_session_revoked, issue_access_token, list_sessions,
    revoke_all_user_sessions, revoke_family_on_reuse, revoke_other_sessions_keeping,
    revoke_refresh_token, revoke_session, revoke_session_tokens, send_account_exists_email,
    send_otp_email, start_registration, store_refresh_token, touch_session, update_user_password,
    update_user_profile, validate_and_rotate_refresh_token, verify_access_token,
    verify_registration, RefreshOutcome, SessionMeta, DUMMY_ARGON2_HASH,
};
use redis::AsyncCommands;
use serde::Deserialize;
use shared::{
    ApiResponse, AppError, ChangePasswordRequest, ErrorPayload, LoginRequest, PasswordChangedDto,
    RefreshTokenRequest, RegistrationStartedDto, SessionDto, SignupRequest, TokenResponse,
    UpdateProfileRequest, UserProfileDto, VerifyOtpRequest,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

const INVALID_CREDENTIALS: &str = "Invalid email or password";

fn redis_unavailable(error: redis::RedisError) -> HttpError {
    tracing::warn!(target: "server::auth", error = %error, "Redis command failed");
    HttpError(AppError::ServiceUnavailable {
        retry_after: infra::REDIS_RETRY_AFTER_SECS,
    })
}

fn now_secs() -> usize {
    Utc::now().timestamp() as usize
}

/// Signs a fresh access token and persists a new refresh token for `user`.
/// Device facts recorded with a new session so the reader can recognise it
/// on the shelf: browser and platform from the User-Agent, a coarse network
/// prefix from the client address.
fn session_meta(state: &AppState, headers: &HeaderMap, jti: Uuid, sid: Uuid) -> SessionMeta {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let ip = client_ip_from_headers(headers, &state.config.server);
    SessionMeta {
        device: domain::device_label(user_agent),
        ip_prefix: domain::ip_prefix(&ip),
        jti,
        sid,
    }
}

async fn issue_token_pair(
    state: &AppState,
    redis: &mut infra::RedisConn,
    user: &users::Model,
    issued_at: usize,
    headers: &HeaderMap,
) -> Result<TokenResponse, HttpError> {
    let sid = Uuid::new_v4();
    let (access_token, jti) = issue_access_token(
        user.id,
        &user.email,
        &user.role,
        state.config.jwt_secret(),
        state.config.auth.access_expiry_minutes,
        issued_at,
        sid,
    )
    .map_err(HttpError::from)?;

    let refresh_token = generate_refresh_token();
    store_refresh_token(
        redis,
        &refresh_token,
        user.id,
        state.config.auth.refresh_expiry_days,
        session_meta(state, headers, jti, sid),
    )
    .await
    .map_err(HttpError::from)?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.auth.access_expiry_minutes as i64 * 60,
        user: Some(user_to_dto(user)),
    })
}

/// Two-layer login brute-force accounting, counted before the password is
/// checked so parallel guesses each take their own slot. The (email, IP)
/// pair allows 5 attempts so hammering a victim's address cannot lock the
/// victim out from their own network; the per-email aggregate caps
/// distributed guessing at 20 per 15 minutes.
const LOGIN_WINDOW_SECS: u64 = 900;
const PAIR_ATTEMPTS: i64 = 5;
const EMAIL_ATTEMPTS: i64 = 20;

fn login_attempt_keys(email: &str, ip: &str) -> [(String, i64); 2] {
    [
        (format!("auth:login:fail:{email}:{ip}"), PAIR_ATTEMPTS),
        (format!("auth:login:fail:{email}"), EMAIL_ATTEMPTS),
    ]
}

/// Counts this attempt and refuses it once either budget is spent.
async fn claim_login_attempt(
    redis: &mut infra::RedisConn,
    email: &str,
    ip: &str,
) -> Result<(), HttpError> {
    for (key, limit) in login_attempt_keys(email, ip) {
        let (attempts, ttl) = hit_window(redis, &key, LOGIN_WINDOW_SECS)
            .await
            .map_err(redis_unavailable)?;
        if attempts > limit {
            return Err(HttpError(AppError::RateLimited {
                retry_after: ttl.max(1) as u64,
            }));
        }
    }
    Ok(())
}

/// A failed attempt that spent a budget locks it for a full window from now.
async fn note_login_failure(redis: &mut infra::RedisConn, email: &str, ip: &str) {
    for (key, limit) in login_attempt_keys(email, ip) {
        let attempts: i64 = redis.get(&key).await.unwrap_or(0);
        if attempts >= limit {
            let _: Result<(), _> = redis.expire(&key, LOGIN_WINDOW_SECS as i64).await;
        }
    }
}

/// Converts a database user model to a public UserProfileDto
fn user_to_dto(user: &users::Model) -> UserProfileDto {
    UserProfileDto {
        id: user.id,
        display_name: user.display_name.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        avatar_url: user.avatar_url.clone(),
        is_active: user.is_active,
        created_at: user.created_at.with_timezone(&Utc),
    }
}

/// True only for the first count past the ceiling, so the operator sees one
/// event per window however long the refusals go on.
fn first_refusal(count: i64, ceiling: i64) -> bool {
    count == ceiling + 1
}

/// Counts one signup mail against the hourly ceiling of every address
/// together; past it signup answers 503 and the first refusal of the hour
/// is logged once for the operator.
async fn claim_mail_budget(redis: &mut infra::RedisConn, budget: u32) -> Result<(), HttpError> {
    if budget == 0 {
        return Ok(());
    }
    let (sent, ttl) = hit_window(redis, "budget:otp:hour", 3600)
        .await
        .map_err(redis_unavailable)?;
    let budget = i64::from(budget);
    if sent <= budget {
        return Ok(());
    }
    if first_refusal(sent, budget) {
        tracing::warn!(target: "server::auth", event = "otp_budget_exhausted", budget, "Hourly signup mail budget spent; signups answer 503 until the hour turns");
    }
    Err(HttpError(AppError::ServiceUnavailable {
        retry_after: ttl.max(1) as u64,
    }))
}

/// Register a new account with email OTP verification
#[utoipa::path(
    post,
    path = "/api/v1/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "Verification OTP sent to email; an address that already has an account answers the same and its owner is told instead", body = ApiResponse<RegistrationStartedDto>),
        (status = 429, description = "Signup cap for this network, codes for this address, or guesses for it spent; Retry-After says when", body = ApiResponse<ErrorPayload>)
    ),
    tag = "Authentication"
)]
pub async fn signup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SignupRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    // Per-IP signup cap: rotating the local part cannot bypass this, so one
    // client cannot burn SMTP quota or sender reputation at will.
    let client_ip = client_subject_from_headers(&headers, &state.config.server);
    let ip_key = format!("otp:signup:ip:{client_ip}");
    let (ip_count, ip_ttl) = hit_window(&mut redis_conn, &ip_key, 3600)
        .await
        .map_err(redis_unavailable)?;
    if ip_count > 10 {
        return Err(HttpError(AppError::RateLimited {
            retry_after: ip_ttl.max(1) as u64,
        }));
    }

    let registered = find_user_by_email(&state.db, &req.email)
        .await
        .map_err(HttpError::from)?
        .is_some_and(|user| user.is_active);

    // Registrations for one address live side by side, so nobody can hold
    // the address of someone else; what stays bounded is the mail sent to
    // it and the guesses spent on it.
    ensure_guesses_left(&mut redis_conn, &req.email)
        .await
        .map_err(HttpError::from)?;
    claim_code_quota(&mut redis_conn, &req.email)
        .await
        .map_err(HttpError::from)?;
    claim_mail_budget(&mut redis_conn, state.config.email.otp_hourly_budget).await?;

    let password_hash = state
        .argon2
        .hash(req.password.clone())
        .await
        .map_err(HttpError::from)?;
    let (registration_id, otp) = start_registration(
        &mut redis_conn,
        &req.email,
        &req.display_name,
        &password_hash,
    )
    .await
    .map_err(HttpError::from)?;

    // An address that already has an account answers like a new one, so
    // signup does not tell who is registered; its owner gets a note instead
    // of a code. Fail closed: a 201 without a delivered message strands the
    // user at the OTP step.
    let delivered = if registered {
        send_account_exists_email(&state.config.email, &req.email).await
    } else {
        send_otp_email(&state.config.email, &req.email, &otp).await
    };
    if let Err(e) = delivered {
        tracing::warn!(target: "server::auth", error = %e, "OTP email delivery failed");
        return Err(HttpError(e));
    }

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(RegistrationStartedDto {
            message: "Verification OTP sent to email".to_string(),
            email: req.email,
            registration_id,
        })),
    )
        .into_response())
}

/// Verify email registration OTP and issue initial tokens
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-otp",
    request_body = VerifyOtpRequest,
    responses(
        (status = 200, description = "Verification successful, JWT tokens issued", body = ApiResponse<TokenResponse>),
        (status = 400, description = "Invalid or expired OTP", body = ApiResponse<ErrorPayload>),
        (status = 409, description = "The address was verified by another registration first", body = ApiResponse<ErrorPayload>),
        (status = 429, description = "Code locked after three misses, or the address spent its daily guesses; Retry-After says when to try again", body = ApiResponse<ErrorPayload>)
    ),
    tag = "Authentication"
)]
pub async fn verify_otp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<VerifyOtpRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    let registration =
        verify_registration(&mut redis_conn, &req.email, req.registration_id, &req.otp)
            .await
            .map_err(HttpError::from)?
            .ok_or_else(|| {
                HttpError(AppError::ValidationError(
                    "Invalid or expired OTP".to_string(),
                ))
            })?;

    let user = activate_registration(
        &state.db,
        &req.email,
        &registration.display_name,
        &registration.password_hash,
    )
    .await
    .map_err(HttpError::from)?;

    let response = issue_token_pair(&state, &mut redis_conn, &user, now_secs(), &headers).await?;
    Ok((StatusCode::OK, Json(ApiResponse::success(response))).into_response())
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authentication successful", body = ApiResponse<TokenResponse>),
        (status = 401, description = "Invalid credentials or inactive account", body = ApiResponse<ErrorPayload>)
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    let client_ip = client_subject_from_headers(&headers, &state.config.server);
    claim_login_attempt(&mut redis_conn, &req.email, &client_ip).await?;

    let user = find_user_by_email(&state.db, &req.email)
        .await
        .map_err(HttpError::from)?;

    // Argon2 runs on every path, against a dummy hash when there is no
    // usable one, before anything branches on the account: unknown, pending
    // and wrong-password answers take the same time and say the same thing.
    let stored_hash = user.as_ref().and_then(|u| u.password_hash.clone());
    let matches = state
        .argon2
        .verify(
            req.password.clone(),
            stored_hash
                .clone()
                .unwrap_or_else(|| DUMMY_ARGON2_HASH.to_string()),
        )
        .await
        .map_err(HttpError::from)?;
    let user = match user {
        Some(user) if matches && stored_hash.is_some() && user.is_active => user,
        _ => {
            note_login_failure(&mut redis_conn, &req.email, &client_ip).await;
            return Err(HttpError(AppError::Unauthorized(
                INVALID_CREDENTIALS.to_string(),
            )));
        }
    };

    let keys: Vec<String> = login_attempt_keys(&req.email, &client_ip)
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    let _: Result<(), _> = redis_conn.del(&keys).await;

    let response = issue_token_pair(&state, &mut redis_conn, &user, now_secs(), &headers).await?;
    Ok((StatusCode::OK, Json(ApiResponse::success(response))).into_response())
}

/// Refresh expired access token with refresh token rotation
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token pair rotated successfully", body = ApiResponse<TokenResponse>),
        (status = 401, description = "Invalid or expired refresh token", body = ApiResponse<ErrorPayload>)
    ),
    tag = "Authentication"
)]
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    let outcome = validate_and_rotate_refresh_token(
        &mut redis_conn,
        &req.refresh_token,
        state.config.auth.refresh_expiry_days,
    )
    .await
    .map_err(HttpError::from)?;

    let (user_id, new_refresh_token, sid) = match outcome {
        RefreshOutcome::Rotated {
            user_id,
            refresh_token,
            sid,
        } => (user_id, refresh_token, sid),
        // A token rotated out before the grace window signals theft: the
        // whole family dies so the attacker's copy goes with the victim's.
        RefreshOutcome::Replayed { user_id } => {
            let access_ttl = state.config.auth.access_expiry_minutes * 60;
            let _ = revoke_family_on_reuse(&mut redis_conn, user_id, access_ttl).await;
            return Err(HttpError(AppError::Unauthorized(
                "Invalid or expired refresh token".to_string(),
            )));
        }
    };

    // A device signed out while its refresh raced the sign-out may still hold
    // a live successor; the session revocation outlives it and refuses it.
    if is_session_revoked(&mut redis_conn, sid)
        .await
        .map_err(HttpError::from)?
    {
        let access_ttl = state.config.auth.access_expiry_minutes * 60;
        let _ = revoke_refresh_token(&mut redis_conn, &new_refresh_token, access_ttl).await;
        return Err(HttpError(AppError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        )));
    }

    let user = find_user_by_id(&state.db, user_id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError(AppError::NotFound("User not found".to_string())))?;

    if !user.is_active {
        return Err(HttpError(AppError::Unauthorized(
            "Account has been deactivated".to_string(),
        )));
    }

    let (access_token, jti) = issue_access_token(
        user.id,
        &user.email,
        &user.role,
        state.config.jwt_secret(),
        state.config.auth.access_expiry_minutes,
        now_secs(),
        sid,
    )
    .map_err(HttpError::from)?;
    touch_session(
        &mut redis_conn,
        &new_refresh_token,
        jti,
        state.config.auth.refresh_expiry_days,
    )
    .await;

    let response = TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: (state.config.auth.access_expiry_minutes as i64 * 60),
        user: Some(user_to_dto(&user)),
    };

    Ok((StatusCode::OK, Json(ApiResponse::success(response))).into_response())
}

/// Logout and invalidate active refresh token session
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Successfully logged out", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Authentication"
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    // The session record names the access token last issued with this
    // refresh token; revoking it blacklists that token for its full lifetime.
    let _ = revoke_refresh_token(
        &mut redis_conn,
        &req.refresh_token,
        state.config.auth.access_expiry_minutes * 60,
    )
    .await;

    // Blacklist the presented access token too, if any: it may be newer than
    // the one the session record names.
    if let Some(auth_val) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth_val.strip_prefix("Bearer ") {
            if let Ok(claims) = verify_access_token(token, state.config.jwt_secret()) {
                let now = Utc::now().timestamp() as usize;
                let remaining = if claims.exp > now {
                    (claims.exp - now) as u64
                } else {
                    0
                };
                let _ = blacklist_access_token(&mut redis_conn, claims.jti, remaining).await;
                if let Some(sid) = claims.sid {
                    let lifetime = state.config.auth.refresh_expiry_days * 86400;
                    let _ = revoke_session_tokens(&mut redis_conn, sid, lifetime).await;
                }
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Successfully logged out"
        }))),
    )
        .into_response())
}

/// Retrieve authenticated user profile
#[utoipa::path(
    get,
    path = "/api/v1/me",
    responses(
        (status = 200, description = "User profile retrieved", body = ApiResponse<UserProfileDto>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Response, HttpError> {
    let user = find_user_by_id(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError(AppError::NotFound("User not found".to_string())))?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(user_to_dto(&user))),
    )
        .into_response())
}

/// Update user profile details
#[utoipa::path(
    patch,
    path = "/api/v1/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = ApiResponse<UserProfileDto>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let updated = update_user_profile(&state.db, auth.id, req.display_name, req.avatar_url)
        .await
        .map_err(HttpError::from)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(user_to_dto(&updated))),
    )
        .into_response())
}

/// Change password for authenticated user
#[utoipa::path(
    put,
    path = "/api/v1/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed; carries a fresh token pair when other sessions were revoked", body = ApiResponse<PasswordChangedDto>),
        (status = 401, description = "Invalid current password", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    if req.current_password == req.new_password {
        return Err(HttpError(AppError::BadRequest(
            "New password must be different from current password".to_string(),
        )));
    }

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    // Five attempts per 15 minutes, counted before Argon2 so parallel tries
    // each take their own slot; a failure that spends them locks the change
    // for 15 minutes from then.
    let attempts_key = format!("auth:pwd_change:fail:{}", auth.id);
    let (attempts, ttl) = hit_window(&mut redis_conn, &attempts_key, 900)
        .await
        .map_err(redis_unavailable)?;
    if attempts > 5 {
        return Err(HttpError(AppError::RateLimited {
            retry_after: ttl.max(1) as u64,
        }));
    }

    let user = find_user_by_id(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError(AppError::NotFound("User not found".to_string())))?;

    let is_valid = match &user.password_hash {
        Some(hash) => state
            .argon2
            .verify(req.current_password.clone(), hash.clone())
            .await
            .map_err(HttpError::from)?,
        None => false,
    };

    if !is_valid {
        if attempts >= 5 {
            let _: Result<(), _> = redis_conn.expire(&attempts_key, 900).await;
        }
        // 403, not 401: the session is fine, the answer was wrong. A 401
        // would send the client through a refresh and a second attempt.
        return Err(HttpError(AppError::Forbidden(
            "Current password is incorrect".to_string(),
        )));
    }
    let _: Result<(), _> = redis_conn.del(&attempts_key).await;

    let new_hash = state
        .argon2
        .hash(req.new_password.clone())
        .await
        .map_err(HttpError::from)?;

    // Revoking other sessions on password change is the safe default: a
    // compromised password must not leave attacker sessions alive. Revoke
    // first, write second (as account deletion does): if revocation fails the
    // old password stays, rather than a new password beside live attacker
    // sessions. The revocation also kills the caller's current pair, so a
    // fresh one is handed back instead of forcing a second login.
    let revoke_sessions = req.revoke_other_sessions.unwrap_or(true);
    if revoke_sessions {
        revoke_all_user_sessions(
            &mut redis_conn,
            auth.id,
            state.config.auth.access_expiry_minutes * 60,
        )
        .await
        .map_err(HttpError::from)?;
    }

    update_user_password(&state.db, auth.id, &new_hash)
        .await
        .map_err(HttpError::from)?;

    let tokens = if revoke_sessions {
        // Stamped one second past the revocation cut, which compares whole
        // seconds; otherwise the fresh pair would be born revoked.
        Some(issue_token_pair(&state, &mut redis_conn, &user, now_secs() + 1, &headers).await?)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(PasswordChangedDto {
            message: "Password changed successfully".to_string(),
            tokens,
        })),
    )
        .into_response())
}

/// Delete account with cascading data deletion
#[utoipa::path(
    delete,
    path = "/api/v1/me",
    responses(
        (status = 200, description = "Account deleted successfully", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn delete_me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Response, HttpError> {
    // Revoke first, delete second: if Redis is unreachable the account must
    // stay (fail closed) rather than leave live tokens on a deleted account.
    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;
    revoke_all_user_sessions(
        &mut redis_conn,
        auth.id,
        state.config.auth.access_expiry_minutes * 60,
    )
    .await
    .map_err(HttpError::from)?;

    delete_user_by_id(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Account successfully deleted"
        }))),
    )
        .into_response())
}

/// Revoke all active sessions and access tokens for the authenticated user
#[utoipa::path(
    post,
    path = "/api/v1/auth/revoke-all",
    responses(
        (status = 200, description = "All active sessions revoked successfully", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "Authentication"
)]
pub async fn revoke_all(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<RevokeAllQuery>,
) -> Result<Response, HttpError> {
    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;
    let access_ttl = state.config.auth.access_expiry_minutes * 60;

    if query.keep_current.unwrap_or(false) {
        let revoked =
            revoke_other_sessions_keeping(&mut redis_conn, auth.id, auth.jti, auth.sid, access_ttl)
                .await
                .map_err(HttpError::from)?;
        return Ok((
            StatusCode::OK,
            Json(ApiResponse::success(serde_json::json!({
                "message": "Other sessions have been revoked",
                "revoked": revoked
            }))),
        )
            .into_response());
    }

    revoke_all_user_sessions(&mut redis_conn, auth.id, access_ttl)
        .await
        .map_err(HttpError::from)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "All sessions have been revoked successfully"
        }))),
    )
        .into_response())
}

/// `keep_current=true` signs out every other device but the caller's.
#[derive(Debug, Deserialize)]
pub struct RevokeAllQuery {
    keep_current: Option<bool>,
}

/// The caller's signed-in devices, most recently seen first.
#[utoipa::path(
    get,
    path = "/api/v1/me/sessions",
    responses(
        (status = 200, description = "Live sessions, the current one flagged", body = Vec<SessionDto>),
        (status = 401, description = "Authentication required")
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn list_my_sessions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Response, HttpError> {
    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;
    let mut sessions: Vec<SessionDto> = list_sessions(&mut redis_conn, auth.id)
        .await
        .into_iter()
        .map(|(id, record)| match record {
            Some(r) => SessionDto {
                id,
                device: r.device,
                ip_prefix: r.ip_prefix,
                created_at: chrono::DateTime::from_timestamp(r.created_at, 0).unwrap_or_default(),
                last_seen_at: chrono::DateTime::from_timestamp(r.last_seen_at, 0)
                    .unwrap_or_default(),
                current: r.jti == auth.jti || Some(r.sid) == auth.sid,
            },
            None => SessionDto {
                id,
                device: String::new(),
                ip_prefix: String::new(),
                created_at: chrono::DateTime::<Utc>::default(),
                last_seen_at: chrono::DateTime::<Utc>::default(),
                current: false,
            },
        })
        .collect();
    sessions.sort_by(|a, b| {
        b.current
            .cmp(&a.current)
            .then(b.last_seen_at.cmp(&a.last_seen_at))
    });
    Ok((StatusCode::OK, Json(ApiResponse::success(sessions))).into_response())
}

/// Signs one device out: its refresh token dies and its last access token is
/// blacklisted. Revoking the current session signs the caller out too.
#[utoipa::path(
    delete,
    path = "/api/v1/me/sessions/{session_id}",
    params(("session_id" = String, Path, description = "Session id from the listing")),
    responses(
        (status = 200, description = "Session revoked"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "No such session for this account")
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn revoke_my_session(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(session_id): Path<String>,
) -> Result<Response, HttpError> {
    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;
    let revoked = revoke_session(
        &mut redis_conn,
        auth.id,
        &session_id,
        state.config.auth.access_expiry_minutes * 60,
    )
    .await
    .map_err(HttpError::from)?;
    if !revoked {
        return Err(HttpError(AppError::NotFound(
            "Session not found".to_string(),
        )));
    }
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({ "revoked": true }))),
    )
        .into_response())
}

/// Assembles public authentication and user management routes
pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signup", post(signup))
        .route("/verify-otp", post(verify_otp))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/revoke-all", post(revoke_all))
}

/// Assembles user profile management routes
pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_me))
        .route("/", patch(update_me))
        .route("/", delete(delete_me))
        .route("/password", put(change_password))
        .route("/sessions", get(list_my_sessions))
        .route("/sessions/{session_id}", delete(revoke_my_session))
        .route("/badges", get(crate::routes::list_user_badges))
        .route("/streak", get(crate::routes::get_my_streak))
}

#[cfg(test)]
mod tests {
    use super::first_refusal;

    #[test]
    fn a_spent_ceiling_is_reported_once_per_window() {
        let reported = (1..=200).filter(|&n| first_refusal(n, 100)).count();
        assert_eq!(reported, 1);
        assert!(first_refusal(101, 100));
    }
}
