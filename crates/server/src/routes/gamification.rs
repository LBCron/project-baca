//! Heartbeat, streak, and badge endpoints.

use crate::{error::HttpError, middleware::auth::AuthUser, AppState};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use shared::{
    ApiResponse, AppError, BadgeDto, ErrorPayload, ReadingHeartbeatRequest,
    ReadingHeartbeatResponse, UserBadgeDto,
};
use std::sync::Arc;
use validator::Validate;

/// Minimum seconds between two rewarded heartbeats for the same user+book.
/// Replay bursts inside the window are rejected so scripts cannot inflate
/// XP, streaks, or badges with cheap 1-second heartbeats.
pub const HEARTBEAT_MIN_INTERVAL_SECS: u64 = 60;

/// Logs reading seconds, evaluates the streak, and awards XP.
#[utoipa::path(
    post,
    path = "/api/v1/activity/heartbeat",
    request_body = ReadingHeartbeatRequest,
    responses(
        (status = 200, description = "Heartbeat recorded and streak evaluated", body = ApiResponse<ReadingHeartbeatResponse>),
        (status = 400, description = "Invalid payload", body = ApiResponse<ErrorPayload>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "Gamification"
)]
pub async fn record_heartbeat(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<ReadingHeartbeatRequest>,
) -> Result<Response, HttpError> {
    req.validate().map_err(HttpError::from)?;

    // Anti-XP-farm gate lives here (route owns Redis; the repository owns
    // Postgres). Redis down fails open to availability: abuse still costs a
    // valid session per hit and the streak math caps daily gains.
    if let Ok(mut redis_conn) = state.get_redis_conn().await {
        use redis::AsyncCommands;
        let farm_key = format!("heartbeat:min_interval:{}:{}", auth.id, req.book_id);
        let fresh: bool = redis_conn.set_nx(&farm_key, "1").await.unwrap_or(true);
        if fresh {
            let _: Result<(), _> = redis_conn
                .expire(&farm_key, HEARTBEAT_MIN_INTERVAL_SECS as i64)
                .await;
        } else {
            let ttl: i64 = redis_conn.ttl(&farm_key).await.unwrap_or(60);
            let retry_after = if ttl > 0 { ttl as u64 } else { 60 };
            return Err(HttpError(AppError::RateLimited { retry_after }));
        }
    }

    let result = infra::record_heartbeat(&state.db, auth.id, &req)
        .await
        .map_err(HttpError::from)?;

    Ok(Json(ApiResponse::success(result)).into_response())
}

/// All master achievement badges.
#[utoipa::path(
    get,
    path = "/api/v1/badges",
    responses(
        (status = 200, description = "List of all master badges", body = ApiResponse<Vec<BadgeDto>>)
    ),
    tag = "Gamification"
)]
pub async fn list_badges(State(state): State<Arc<AppState>>) -> Result<Response, HttpError> {
    let badges = infra::list_badges(&state.db)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(badges)).into_response())
}

/// Badges unlocked by the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/me/badges",
    responses(
        (status = 200, description = "List of user unlocked badges", body = ApiResponse<Vec<UserBadgeDto>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<ErrorPayload>)
    ),
    security(("BearerAuth" = [])),
    tag = "Gamification"
)]
pub async fn list_user_badges(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Response, HttpError> {
    let user_badges = infra::list_user_badges(&state.db, auth.id)
        .await
        .map_err(HttpError::from)?;
    Ok(Json(ApiResponse::success(user_badges)).into_response())
}

pub fn gamification_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/activity/heartbeat", post(record_heartbeat))
        .route("/badges", get(list_badges))
}
