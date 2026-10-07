//! Auth and user endpoints.

use crate::{
    error::HttpError,
    middleware::{auth::AuthUser, rate_limit::client_ip_from_headers},
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
    activate_user_by_email, blacklist_access_token, create_inactive_user, delete_user_by_id,
    entities::users, find_user_by_email, find_user_by_id, generate_and_store_otp,
    generate_refresh_token, hash_password_async, issue_access_token, list_sessions,
    revoke_all_user_sessions, revoke_family_on_reuse, revoke_other_sessions_keeping,
    revoke_refresh_token, revoke_session, send_otp_email, store_refresh_token, touch_session,
    update_inactive_credentials, update_user_password, update_user_profile,
    validate_and_rotate_refresh_token, verify_access_token, verify_and_consume_otp,
    verify_password_async, RefreshOutcome, SessionMeta, DUMMY_ARGON2_HASH,
};
use redis::AsyncCommands;
use serde::Deserialize;
use shared::{
    ApiResponse, AppError, ChangePasswordRequest, ErrorPayload, LoginRequest, PasswordChangedDto,
    RefreshTokenRequest, SessionDto, SignupRequest, TokenResponse, UpdateProfileRequest,
    UserProfileDto, VerifyOtpRequest,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

const INVALID_CREDENTIALS: &str = "Invalid email or password";

fn now_secs() -> usize {
    Utc::now().timestamp() as usize
}

/// Signs a fresh access token and persists a new refresh token for `user`.
/// Device facts recorded with a new session so the reader can recognise it
/// on the shelf: browser and platform from the User-Agent, a coarse network
/// prefix from the client address.
fn session_meta(state: &AppState, headers: &HeaderMap, jti: Uuid) -> SessionMeta {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let ip = client_ip_from_headers(headers, state.config.server.trust_proxy_headers);
    SessionMeta {
        device: domain::device_label(user_agent),
        ip_prefix: domain::ip_prefix(&ip),
        jti,
    }
}

async fn issue_token_pair(
    state: &AppState,
    redis: &mut redis::aio::MultiplexedConnection,
    user: &users::Model,
    issued_at: usize,
    headers: &HeaderMap,
) -> Result<TokenResponse, HttpError> {
    let (access_token, jti) = issue_access_token(
        user.id,
        &user.email,
        &user.role,
        state.config.jwt_secret(),
        state.config.auth.access_expiry_minutes,
        issued_at,
    )
    .map_err(HttpError::from)?;

    let refresh_token = generate_refresh_token();
    store_refresh_token(
        redis,
        &refresh_token,
        user.id,
        state.config.auth.refresh_expiry_days,
        session_meta(state, headers, jti),
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

/// Two-layer login brute-force accounting. The (email, IP) pair locks after
/// 5 failures so hammering a victim's address cannot lock the victim out
/// from their own network; the per-email aggregate caps distributed guessing
/// at 20 failures per 15 minutes.
async fn note_login_failure(redis: &mut redis::aio::MultiplexedConnection, email: &str, ip: &str) {
    let pair_fail_key = format!("auth:login:fail:{email}:{ip}");
    let pair_count: Result<i64, _> = redis.incr(&pair_fail_key, 1).await;
    if let Ok(c) = pair_count {
        let _: Result<(), _> = redis.expire(&pair_fail_key, 900).await;
        if c >= 5 {
            let pair_lock_key = format!("auth:login:lockout:{email}:{ip}");
            let _: Result<(), _> = redis.set_ex(&pair_lock_key, "1", 900).await;
        }
    }
    let email_fail_key = format!("auth:login:fail:{email}");
    let email_count: Result<i64, _> = redis.incr(&email_fail_key, 1).await;
    if let Ok(c) = email_count {
        let _: Result<(), _> = redis.expire(&email_fail_key, 900).await;
        if c >= 20 {
            let email_lock_key = format!("auth:login:lockout:{email}");
            let _: Result<(), _> = redis.set_ex(&email_lock_key, "1", 900).await;
        }
    }
}

/// Returns the lockout retry delay when the pair or aggregate lock is set.
async fn login_lockout_retry_after(
    redis: &mut redis::aio::MultiplexedConnection,
    email: &str,
    ip: &str,
) -> Option<u64> {
    for key in [
        format!("auth:login:lockout:{email}:{ip}"),
        format!("auth:login:lockout:{email}"),
    ] {
        let locked: bool = redis.exists(&key).await.unwrap_or(false);
        if locked {
            let ttl: i64 = redis.ttl(&key).await.unwrap_or(900);
            return Some(if ttl > 0 { ttl as u64 } else { 900 });
        }
    }
    None
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

/// Register a new account with email OTP verification
#[utoipa::path(
    post,
    path = "/api/v1/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "Verification OTP sent to email", body = ApiResponse<serde_json::Value>),
        (status = 409, description = "Email already registered", body = ApiResponse<ErrorPayload>)
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

    // Anti-Email-Bombing: 60-second cooldown per target email
    let cooldown_key = format!("otp:cooldown:{}", req.email);
    let in_cooldown: bool = redis_conn.exists(&cooldown_key).await.unwrap_or(false);
    if in_cooldown {
        let ttl: i64 = redis_conn.ttl(&cooldown_key).await.unwrap_or(60);
        let retry_after = if ttl > 0 { ttl as u64 } else { 60 };
        return Err(HttpError(AppError::RateLimited { retry_after }));
    }

    // Per-IP signup cap: rotating the local part cannot bypass this, so one
    // client cannot burn SMTP quota or sender reputation at will.
    let client_ip = client_ip_from_headers(&headers, state.config.server.trust_proxy_headers);
    let ip_key = format!("otp:signup:ip:{client_ip}");
    let ip_count: Result<i64, _> = redis_conn.incr(&ip_key, 1).await;
    if let Ok(c) = ip_count {
        let _: Result<(), _> = redis_conn.expire(&ip_key, 3600).await;
        if c > 10 {
            return Err(HttpError(AppError::RateLimited { retry_after: 3600 }));
        }
    }

    let existing_user = find_user_by_email(&state.db, &req.email)
        .await
        .map_err(HttpError::from)?;

    let password_hash = hash_password_async(req.password.clone())
        .await
        .map_err(HttpError::from)?;

    if let Some(user) = existing_user {
        if user.is_active {
            return Err(HttpError(AppError::Conflict(
                "An account with this email is already registered".to_string(),
            )));
        }

        // Unverified user requesting new registration: update credentials
        update_inactive_credentials(&state.db, user, &req.display_name, &password_hash)
            .await
            .map_err(HttpError::from)?;
    } else {
        // Create new inactive user awaiting OTP verification
        create_inactive_user(&state.db, &req.email, &req.display_name, &password_hash)
            .await
            .map_err(HttpError::from)?;
    }

    let otp = generate_and_store_otp(&mut redis_conn, &req.email)
        .await
        .map_err(HttpError::from)?;

    // Set 60-second cooldown for subsequent OTP requests for this email
    let _: Result<(), _> = redis_conn.set_ex(&cooldown_key, "1", 60).await;

    // Fail closed: a 201 without a deliverable code strands the user at the
    // OTP step, and the cooldown must not block an immediate retry.
    if let Err(e) = send_otp_email(&state.config.email, &req.email, &otp).await {
        let _: Result<(), _> = redis_conn.del(&cooldown_key).await;
        tracing::warn!(target: "server::auth", error = %e, "OTP email delivery failed");
        return Err(HttpError(e));
    }

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Verification OTP sent to email",
            "email": req.email
        }))),
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
        (status = 429, description = "Code locked after three misses; Retry-After says when to try again", body = ApiResponse<ErrorPayload>)
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

    let is_valid = verify_and_consume_otp(&mut redis_conn, &req.email, &req.otp)
        .await
        .map_err(HttpError::from)?;

    if !is_valid {
        return Err(HttpError(AppError::ValidationError(
            "Invalid or expired OTP".to_string(),
        )));
    }

    let updated_user = activate_user_by_email(&state.db, &req.email)
        .await
        .map_err(HttpError::from)?;

    let response =
        issue_token_pair(&state, &mut redis_conn, &updated_user, now_secs(), &headers).await?;
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

    let client_ip = client_ip_from_headers(&headers, state.config.server.trust_proxy_headers);

    // Lockout check: pair (email, IP) lock after 5 failures, aggregate email
    // lock after 20 — 15 minutes each.
    if let Some(retry_after) =
        login_lockout_retry_after(&mut redis_conn, &req.email, &client_ip).await
    {
        return Err(HttpError(AppError::RateLimited { retry_after }));
    }

    let user = find_user_by_email(&state.db, &req.email)
        .await
        .map_err(HttpError::from)?;

    let user = match user {
        Some(u) => u,
        None => {
            // Dummy verification defeats timing-based user enumeration.
            let _ =
                verify_password_async(req.password.clone(), DUMMY_ARGON2_HASH.to_string()).await;

            note_login_failure(&mut redis_conn, &req.email, &client_ip).await;
            return Err(HttpError(AppError::Unauthorized(
                INVALID_CREDENTIALS.to_string(),
            )));
        }
    };

    // An unverified account answers exactly like a wrong password, so the
    // activation state of an address cannot be probed.
    if !user.is_active {
        note_login_failure(&mut redis_conn, &req.email, &client_ip).await;
        return Err(HttpError(AppError::Unauthorized(
            INVALID_CREDENTIALS.to_string(),
        )));
    }

    let is_valid = match &user.password_hash {
        Some(hash) => verify_password_async(req.password.clone(), hash.clone())
            .await
            .map_err(HttpError::from)?,
        None => false,
    };

    if !is_valid {
        note_login_failure(&mut redis_conn, &req.email, &client_ip).await;
        return Err(HttpError(AppError::Unauthorized(
            INVALID_CREDENTIALS.to_string(),
        )));
    }

    // Reset login failure counters on successful authentication
    let pair_fail_key = format!("auth:login:fail:{}:{}", req.email, client_ip);
    let pair_lock_key = format!("auth:login:lockout:{}:{}", req.email, client_ip);
    let email_fail_key = format!("auth:login:fail:{}", req.email);
    let email_lock_key = format!("auth:login:lockout:{}", req.email);
    let _: Result<(), _> = redis_conn
        .del(&[
            &pair_fail_key,
            &pair_lock_key,
            &email_fail_key,
            &email_lock_key,
        ])
        .await;

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

    let (user_id, new_refresh_token) = match outcome {
        RefreshOutcome::Rotated {
            user_id,
            refresh_token,
        } => (user_id, refresh_token),
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

    let _ = revoke_refresh_token(&mut redis_conn, &req.refresh_token).await;

    // Blacklist the access token until its expiry when a header is present.
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

    // Throttling: Lockout after 5 failed password change attempts
    let lockout_key = format!("auth:pwd_change:lockout:{}", auth.id);
    let is_locked: bool = redis_conn.exists(&lockout_key).await.unwrap_or(false);
    if is_locked {
        let ttl: i64 = redis_conn.ttl(&lockout_key).await.unwrap_or(900);
        let retry_after = if ttl > 0 { ttl as u64 } else { 900 };
        return Err(HttpError(AppError::RateLimited { retry_after }));
    }

    let user = find_user_by_id(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError(AppError::NotFound("User not found".to_string())))?;

    let is_valid = match &user.password_hash {
        Some(hash) => verify_password_async(req.current_password.clone(), hash.clone())
            .await
            .map_err(HttpError::from)?,
        None => false,
    };

    if !is_valid {
        let fail_key = format!("auth:pwd_change:fail:{}", auth.id);
        let count: Result<i64, _> = redis_conn.incr(&fail_key, 1).await;
        if let Ok(c) = count {
            let _: Result<(), _> = redis_conn.expire(&fail_key, 900).await;
            if c >= 5 {
                let _: Result<(), _> = redis_conn.set_ex(&lockout_key, "1", 900).await;
            }
        }
        return Err(HttpError(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        )));
    }

    // Reset password change failure counter on success
    let fail_key = format!("auth:pwd_change:fail:{}", auth.id);
    let _: Result<(), _> = redis_conn.del(&[&fail_key, &lockout_key]).await;

    let new_hash = hash_password_async(req.new_password.clone())
        .await
        .map_err(HttpError::from)?;

    update_user_password(&state.db, auth.id, &new_hash)
        .await
        .map_err(HttpError::from)?;

    // Revoking other sessions on password change is the safe default: a
    // compromised password must not leave attacker sessions alive. The
    // revocation also kills the caller's current pair, so a fresh one is
    // handed back instead of forcing a second login.
    let tokens = if req.revoke_other_sessions.unwrap_or(true) {
        revoke_all_user_sessions(
            &mut redis_conn,
            auth.id,
            state.config.auth.access_expiry_minutes * 60,
        )
        .await
        .map_err(HttpError::from)?;
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
        let revoked = revoke_other_sessions_keeping(&mut redis_conn, auth.id, auth.jti, access_ttl)
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
                current: r.jti == auth.jti,
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
