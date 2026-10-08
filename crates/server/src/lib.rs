//! Axum router, state, middleware, and OpenAPI spec.

pub mod error;
pub mod middleware;
pub mod routes;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, State},
    http::{header, HeaderName, HeaderValue, Request, Response, StatusCode},
    middleware as axum_mw,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use infra::{AppConfig, EmbeddingProvider, StorageService};
use sea_orm::DatabaseConnection;
use shared::*;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: redis::Client,
    pub redis_conn: Option<redis::aio::MultiplexedConnection>,
    pub config: Arc<AppConfig>,
    /// Dual-mode embedding provider (Gemini REST API or FastEmbed CPU).
    /// Wrapped in Arc so cloning AppState does not clone the provider internals.
    pub embedding: Arc<dyn EmbeddingProvider>,
    /// S3-compatible object storage. `None` when MinIO/S3 is unreachable at
    /// startup; upload endpoints fail closed with a 500 in that case.
    pub storage: Option<Arc<StorageService>>,
}

impl AppState {
    /// Returns a multiplexed Redis connection handle.
    /// If pre-initialized, clones the handle in sub-microseconds without opening a new TCP socket.
    /// Falls back to establishing a new multiplexed connection if not pre-initialized.
    pub async fn get_redis_conn(&self) -> Result<redis::aio::MultiplexedConnection, AppError> {
        if let Some(conn) = &self.redis_conn {
            Ok(conn.clone())
        } else {
            self.redis
                .get_multiplexed_tokio_connection()
                .await
                .map_err(|e| AppError::Internal(format!("Redis connection failed: {e}")))
        }
    }
}

/// OpenAPI documentation root schema
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        api_health_check,
        routes::auth::signup,
        routes::auth::verify_otp,
        routes::auth::login,
        routes::auth::list_my_sessions,
        routes::auth::revoke_my_session,
        routes::auth::refresh,
        routes::auth::logout,
        routes::auth::revoke_all,
        routes::auth::get_me,
        routes::auth::update_me,
        routes::auth::change_password,
        routes::auth::delete_me,
        routes::progress::get_active_progress,
        routes::progress::update_progress,
        routes::progress::merge_guest_progress,
        routes::books::list_books,
        routes::books::search_books,
        routes::books::get_book,
        routes::books::get_chapter,
        routes::books::get_offline_bundle,
        routes::covers::get_cover,
        routes::gamification::record_heartbeat,
        routes::gamification::list_badges,
        routes::gamification::list_user_badges,
        routes::gamification::get_my_streak,
        routes::quotes::search_book_quotes,
        routes::quotes::handle_save_quote,
        routes::quotes::handle_save_quotes_batch,
        routes::quotes::handle_list_saved_quotes,
        routes::quotes::get_quote_card,
        routes::insights::get_atomic_cards,
        routes::insights::get_chapter_recap,
        routes::admin::upload_book,
        routes::admin::ingestion_status,
        routes::admin::dropoff_analytics,
        routes::admin::list_catalog,
        routes::admin::patch_book,
        routes::admin::list_dlq,
        routes::admin::replay_dlq,
    ),
    components(
        schemas(
            ErrorPayload,
            SignupRequest,
            VerifyOtpRequest,
            LoginRequest,
            RefreshTokenRequest,
            TokenResponse,
            UserProfileDto,
            SessionDto,
            UpdateProfileRequest,
            ChangePasswordRequest,
            PasswordChangedDto,
            GuestProgressRecord,
            GuestMergeRequest,
            BookSummaryDto,
            BookDetailDto,
            ChapterSummaryDto,
            ChapterDetailDto,
            OfflineBundleDto,
            BookSearchResultDto,
            ReadingProgressUpdateDto,
            ActiveProgressDto,
            ReadingHeartbeatRequest,
            ReadingHeartbeatResponse,
            ReadingStreakDto,
            BadgeDto,
            UserBadgeDto,
            QuoteSearchRequest,
            QuoteSearchResultDto,
            AtomicCardsDto,
            ChapterRecapDto,
            SaveQuoteRequest,
            SavedQuoteResponseDto,
            SaveQuotesBatchRequest,
            SaveQuoteOutcomeDto,
            SaveQuotesBatchResponseDto,
            UploadBookResponseDto,
            JobStatusDto,
            AdminBookRowDto,
            AdminBookPatchRequest,
            DlqEntryDto,
            DlqReplayResponseDto,
            DropOffPointDto
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "System & Health", description = "Runtime health checks and infrastructure connectivity"),
        (name = "Authentication", description = "User registration, OTP verification, and JWT sessions"),
        (name = "User Management", description = "Profile updates, password management, and account deletion"),
        (name = "Catalog", description = "Public domain book catalog, FTS search, and chapter reader"),
        (name = "Reading Progress", description = "Progress tracking, CFI anchors, and guest reconciliation"),
        (name = "Gamification", description = "Reading streaks, heartbeats, and achievement badges"),
        (name = "Semantic Search", description = "HNSW pgvector quote search and AI-powered atomic insight cards"),
        (name = "Administration", description = "Admin EPUB ingestion, job monitoring, and retention analytics")
    ),
    info(
        title = "Project Baca REST API",
        version = "0.1.0",
        description = "Public Domain Classic E-Reader & Atomic Insights API.",
        license(name = "MIT OR Apache-2.0")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Middleware for request ID correlation (x-request-id) across tracing spans and HTTP responses
pub async fn request_id_middleware(req: Request<Body>, next: axum_mw::Next) -> Response<Body> {
    // Server-generated IDs are never trusted from the client: a caller-supplied
    // id could collide or be used to probe correlation, so always mint fresh.
    let request_id = uuid::Uuid::new_v4().to_string();
    let Ok(request_id_value) = HeaderValue::from_str(&request_id) else {
        return next.run(req).await;
    };

    let span = tracing::info_span!("http_request", request_id = %request_id);
    let _guard = span.enter();

    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER.clone(), request_id_value);
    response
}

/// Middleware injecting OWASP recommended security headers across all HTTP responses
pub async fn security_headers_middleware(
    req: Request<Body>,
    next: axum_mw::Next,
) -> Response<Body> {
    // Swagger UI ships inline scripts and styles, so its document-only CSP
    // allows 'unsafe-inline' + same-origin assets. Everywhere else keeps the
    // strict default ('none') so API/JSON/SVG responses cannot run scripts.
    let is_swagger =
        req.uri().path().starts_with("/swagger-ui") || req.uri().path().starts_with("/api-docs");
    let csp = if is_swagger {
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'"
    } else {
        "default-src 'none'; frame-ancestors 'none'; base-uri 'none'"
    };
    // Tokens, profiles, and anything answered to a bearer must never land in
    // a shared cache; a handler that sets its own policy (public assets) wins.
    let private = req.uri().path().starts_with("/api/v1/auth")
        || req.headers().contains_key(header::AUTHORIZATION);
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    if private {
        headers
            .entry(header::CACHE_CONTROL)
            .or_insert(HeaderValue::from_static("no-store"));
    }
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("0"),
    );
    if let Ok(v) = HeaderValue::from_str(csp) {
        headers.insert(HeaderName::from_static("content-security-policy"), v);
    }
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    // Public images opt out per response (the reader may load covers from
    // another origin in development); everything else stays same-origin.
    headers
        .entry(HeaderName::from_static("cross-origin-resource-policy"))
        .or_insert(HeaderValue::from_static("same-origin"));
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    // Browsers ignore HSTS over plain HTTP, so emitting it unconditionally is
    // safe for the dev stack and binding once TLS terminates at the edge.
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    response
}

/// Liveness probe. A process that booted without a database answers 503 so
/// the orchestrator restarts it instead of routing readers to it.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is up with its database connection"),
        (status = 503, description = "Server is up but has no database connection")
    ),
    tag = "System & Health"
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_attached = !matches!(state.db, DatabaseConnection::Disconnected);
    let (http_status, status) = if db_attached {
        (StatusCode::OK, "healthy")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded")
    };
    (
        http_status,
        Json(ApiResponse::success(serde_json::json!({
            "service": "project-baca-server",
            "status": status,
            "version": "0.1.0"
        }))),
    )
}

/// Infrastructure health check handler for database and cache connectivity
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Database and cache connectivity status")
    ),
    tag = "System & Health"
)]
pub async fn api_health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_connected = state.db.ping().await.is_ok();
    let redis_connected = match state.get_redis_conn().await {
        Ok(mut conn) => redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map(|pong| pong == "PONG")
            .unwrap_or(false),
        Err(_) => false,
    };
    let degraded = !(db_connected && redis_connected);

    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "status": if degraded { "degraded" } else { "ok" },
            "postgres": if db_connected { "connected" } else { "disconnected" },
            "redis": if redis_connected { "connected" } else { "disconnected" }
        }))),
    )
}

/// Global fallback handler for unmatched routes, returning standardized JSON error
pub async fn not_found_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::<()>::error(
            "NOT_FOUND",
            "The requested resource was not found",
            None,
        )),
    )
}

/// Swagger UI and OpenAPI JSON. Not mounted at all in production: the spec
/// fingerprinting surface stays available in dev/staging only.
fn swagger_routes(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    if state.config.is_production() {
        return Router::new();
    }
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Assembles Axum Router with Swagger UI, sub-routers, and middleware pipeline
pub fn create_app(state: Arc<AppState>) -> Router {
    // Browsers only: origins, methods, and headers restricted. Any other
    // origin gets no ACAO header, so credentialed cross-site reads fail closed.
    let allowed_origins = state.config.server.cors_allowed_origins.clone();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
            origin
                .to_str()
                .map(|s| allowed_origins.iter().any(|o| o == s))
                .unwrap_or(false)
        }))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            REQUEST_ID_HEADER.clone(),
        ]);

    let auth_router = routes::auth_routes().layer(axum_mw::from_fn_with_state(
        state.clone(),
        crate::middleware::rate_limit_middleware,
    ));

    // AI quote search endpoints: apply per-user AI rate limit (10 req/min via Redis)
    let quotes_router = routes::quotes_routes().layer(axum_mw::from_fn_with_state(
        state.clone(),
        crate::middleware::ai_rate_limit_middleware,
    ));

    // Unauthenticated reads get a per-IP cap only when configured: the dev
    // stack and in-memory tests share one fallback IP and would collide.
    let public_cap = state.config.server.public_rate_limit_per_minute;
    let public = |router: Router<Arc<AppState>>| {
        if public_cap > 0 {
            router.layer(axum_mw::from_fn_with_state(
                state.clone(),
                crate::middleware::public_rate_limit_middleware,
            ))
        } else {
            router
        }
    };

    // Multipart EPUB uploads need more than axum's 2 MB default; the handler
    // still enforces the 50 MB product limit while streaming.
    let admin_router = routes::admin_routes().layer(DefaultBodyLimit::max(
        routes::admin::MAX_EPUB_BYTES + 64 * 1024,
    ));

    Router::new()
        .merge(swagger_routes(&state))
        .route("/health", get(health_check))
        .route("/api/v1/health", get(api_health_check))
        .nest("/api/v1/auth", auth_router)
        .nest("/api/v1/me", routes::user_routes())
        .nest("/api/v1/books", public(routes::books_routes()))
        .nest("/api/v1/covers", public(routes::covers_routes()))
        .nest("/api/v1/progress", routes::progress_routes())
        .nest("/api/v1", public(routes::gamification_routes()))
        .nest("/api/v1/books", quotes_router)
        .nest("/api/v1/books", public(routes::insights_routes()))
        .nest("/api/v1/quotes", routes::saved_quotes_routes())
        .nest("/api/v1/admin", admin_router)
        .fallback(not_found_handler)
        .layer(axum_mw::from_fn(security_headers_middleware))
        .layer(axum_mw::from_fn(request_id_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
