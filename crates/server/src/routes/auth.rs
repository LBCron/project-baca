//! Auth and user endpoints.

use crate::{error::HttpError, middleware::auth::AuthUser, AppState};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use chrono::Utc;
use infra::{
    activate_user_by_email, blacklist_access_token, create_inactive_user, delete_user_by_id,
    entities::users, find_user_by_email, find_user_by_id, generate_access_token,
    generate_and_store_otp, generate_refresh_token, hash_password_async, revoke_all_user_sessions,
    revoke_refresh_token, send_otp_email, store_refresh_token, update_inactive_credentials,
    update_user_password, update_user_profile, validate_and_rotate_refresh_token,
    verify_access_token, verify_and_consume_otp, verify_password_async, DUMMY_ARGON2_HASH,
};
use redis::AsyncCommands;
use shared::{
    ApiResponse, AppError, ChangePasswordRequest, ErrorPayload, LoginRequest, RefreshTokenRequest,
    SignupRequest, TokenResponse, UpdateProfileRequest, UserProfileDto, VerifyOtpRequest,
};
use std::sync::Arc;
use validator::Validate;

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

    let _ = send_otp_email(&state.config.email, &req.email, &otp).await;

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
        (status = 400, description = "Invalid or expired OTP", body = ApiResponse<ErrorPayload>)
    ),
    tag = "Authentication"
)]
pub async fn verify_otp(
    State(state): State<Arc<AppState>>,
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

    let access_token = generate_access_token(
        updated_user.id,
        &updated_user.email,
        &updated_user.role,
        state.config.jwt_secret(),
        state.config.auth.access_expiry_minutes,
    )
    .map_err(HttpError::from)?;

    let refresh_token = generate_refresh_token();
    store_refresh_token(
        &mut redis_conn,
        &refresh_token,
        updated_user.id,
        state.config.auth.refresh_expiry_days,
    )
    .await
    .map_err(HttpError::from)?;

    let response = TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: (state.config.auth.access_expiry_minutes as i64 * 60),
        user: Some(user_to_dto(&updated_user)),
    };

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
    Json(req): Json<LoginRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    // Lockout check: 15 minutes after 5 failed attempts.
    let lockout_key = format!("auth:login:lockout:{}", req.email);
    let is_locked: bool = redis_conn.exists(&lockout_key).await.unwrap_or(false);
    if is_locked {
        let ttl: i64 = redis_conn.ttl(&lockout_key).await.unwrap_or(900);
        let retry_after = if ttl > 0 { ttl as u64 } else { 900 };
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

            let fail_key = format!("auth:login:fail:{}", req.email);
            let count: Result<i64, _> = redis_conn.incr(&fail_key, 1).await;
            if let Ok(c) = count {
                let _: Result<(), _> = redis_conn.expire(&fail_key, 900).await;
                if c >= 5 {
                    let _: Result<(), _> = redis_conn.set_ex(&lockout_key, "1", 900).await;
                }
            }
            return Err(HttpError(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            )));
        }
    };

    if !user.is_active {
        return Err(HttpError(AppError::Unauthorized(
            "Account not activated. Please verify OTP first.".to_string(),
        )));
    }

    let is_valid = match &user.password_hash {
        Some(hash) => verify_password_async(req.password.clone(), hash.clone())
            .await
            .map_err(HttpError::from)?,
        None => false,
    };

    if !is_valid {
        let fail_key = format!("auth:login:fail:{}", req.email);
        let count: Result<i64, _> = redis_conn.incr(&fail_key, 1).await;
        if let Ok(c) = count {
            let _: Result<(), _> = redis_conn.expire(&fail_key, 900).await;
            if c >= 5 {
                let _: Result<(), _> = redis_conn.set_ex(&lockout_key, "1", 900).await;
            }
        }
        return Err(HttpError(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        )));
    }

    // Reset login failure counters on successful authentication
    let fail_key = format!("auth:login:fail:{}", req.email);
    let _: Result<(), _> = redis_conn.del(&[&fail_key, &lockout_key]).await;

    let access_token = generate_access_token(
        user.id,
        &user.email,
        &user.role,
        state.config.jwt_secret(),
        state.config.auth.access_expiry_minutes,
    )
    .map_err(HttpError::from)?;

    let refresh_token = generate_refresh_token();
    store_refresh_token(
        &mut redis_conn,
        &refresh_token,
        user.id,
        state.config.auth.refresh_expiry_days,
    )
    .await
    .map_err(HttpError::from)?;

    let response = TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: (state.config.auth.access_expiry_minutes as i64 * 60),
        user: Some(user_to_dto(&user)),
    };

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

    let (user_id, new_refresh_token) = validate_and_rotate_refresh_token(
        &mut redis_conn,
        &req.refresh_token,
        state.config.auth.refresh_expiry_days,
    )
    .await
    .map_err(HttpError::from)?;

    let user = find_user_by_id(&state.db, user_id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError(AppError::NotFound("User not found".to_string())))?;

    if !user.is_active {
        return Err(HttpError(AppError::Unauthorized(
            "Account has been deactivated".to_string(),
        )));
    }

    let access_token = generate_access_token(
        user.id,
        &user.email,
        &user.role,
        state.config.jwt_secret(),
        state.config.auth.access_expiry_minutes,
    )
    .map_err(HttpError::from)?;

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
        (status = 200, description = "Password changed successfully", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Invalid current password", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "User Management"
)]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
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
    // compromised password must not leave attacker sessions alive.
    if req.revoke_other_sessions.unwrap_or(true) {
        let _ = revoke_all_user_sessions(
            &mut redis_conn,
            auth.id,
            state.config.auth.access_expiry_minutes * 60,
        )
        .await;
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Password changed successfully"
        }))),
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
    delete_user_by_id(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?;

    // Revoke all active sessions and access tokens in Redis
    if let Ok(mut redis_conn) = state.get_redis_conn().await {
        let _ = revoke_all_user_sessions(
            &mut redis_conn,
            auth.id,
            state.config.auth.access_expiry_minutes * 60,
        )
        .await;
    }

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
) -> Result<Response, HttpError> {
    let mut redis_conn = state.get_redis_conn().await.map_err(HttpError::from)?;

    revoke_all_user_sessions(
        &mut redis_conn,
        auth.id,
        state.config.auth.access_expiry_minutes * 60,
    )
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
        .route("/badges", get(crate::routes::list_user_badges))
}
