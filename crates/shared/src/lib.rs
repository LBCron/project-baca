//! Shared DTOs, Error models, and validation logic for Project Baca.
//! This crate is utilized by both backend Axum and frontend Leptos WASM.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorPayload>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &str, message: &str, details: Option<serde_json::Value>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorPayload {
                code: code.to_string(),
                message: message.to_string(),
                details,
            }),
        }
    }
}

/// Standardized error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Common application errors
#[derive(Debug, thiserror::Error, Serialize, Deserialize, Clone)]
pub enum AppError {
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Validation failed: {message}")]
    ValidationDetailed {
        message: String,
        details: serde_json::Value,
    },

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

// -----------------------------------------------------------------------------
// Authentication & User DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SignupRequest {
    #[serde(alias = "nickname")]
    #[validate(length(
        min = 2,
        max = 100,
        message = "Display name must be between 2 and 100 characters"
    ))]
    pub display_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, max = 128, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VerifyOtpRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(equal = 6, message = "OTP must be exactly 6 digits"))]
    pub otp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token must not be empty"))]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserProfileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserProfileDto {
    pub id: Uuid,
    #[serde(alias = "nickname")]
    pub display_name: String,
    pub email: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Display name must be between 2 and 100 characters"
    ))]
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    #[validate(length(
        min = 8,
        max = 128,
        message = "New password must be at least 8 characters"
    ))]
    pub new_password: String,
    #[serde(default)]
    pub revoke_other_sessions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GuestProgressRecord {
    pub book_id: Uuid,
    pub last_chapter_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub last_anchor_cfi: String,
    #[validate(range(min = 0.0, max = 100.0))]
    pub completion_percentage: f32,
    pub is_finished: Option<bool>,
    pub last_read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GuestMergeRequest {
    #[validate(nested)]
    pub records: Vec<GuestProgressRecord>,
}

// -----------------------------------------------------------------------------
// Catalog & Book DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BookSummaryDto {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub language: String,
    pub primary_theme: String,
    pub sub_theme: Option<String>,
    pub description: String,
    pub cover_url: String,
    pub total_words: i32,
    pub estimated_reading_minutes: i32,
    pub publication_year: Option<i32>,
    pub license: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BookDetailDto {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub language: String,
    pub primary_theme: String,
    pub sub_theme: Option<String>,
    pub description: String,
    pub cover_url: String,
    pub total_words: i32,
    pub estimated_reading_minutes: i32,
    pub source_name: String,
    pub source_url: Option<String>,
    pub license: String,
    pub publication_year: Option<i32>,
    pub status: String,
    pub tags: Vec<String>,
    pub chapters: Vec<ChapterSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChapterSummaryDto {
    pub id: Uuid,
    pub chapter_number: i32,
    pub title: String,
    pub word_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChapterDetailDto {
    pub id: Uuid,
    pub book_id: Uuid,
    pub chapter_number: i32,
    pub title: String,
    pub word_count: i32,
    pub html_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OfflineBundleDto {
    pub book: BookDetailDto,
    pub chapters: Vec<ChapterDetailDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct BookCatalogQuery {
    pub cursor: Option<Uuid>,
    pub limit: Option<u64>,
    pub language: Option<String>,
    pub theme: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct BookSearchQuery {
    pub q: String,
    pub language: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BookSearchResultDto {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub language: String,
    pub primary_theme: String,
    pub cover_url: String,
    pub estimated_reading_minutes: i32,
    pub score: f32,
}

// -----------------------------------------------------------------------------
// Reading Progress & CFI DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReadingProgressUpdateDto {
    pub chapter_id: Uuid,
    #[serde(alias = "cfi_position")]
    #[validate(length(
        min = 1,
        max = 255,
        message = "CFI position must be between 1 and 255 characters"
    ))]
    pub last_anchor_cfi: String,
    #[serde(alias = "percentage")]
    #[validate(range(
        min = 0.0,
        max = 100.0,
        message = "Percentage must be between 0.0 and 100.0"
    ))]
    pub completion_percentage: f32,
}

pub type UpdateProgressRequest = ReadingProgressUpdateDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActiveProgressDto {
    pub book_id: Uuid,
    pub book_title: String,
    pub book_author: String,
    pub book_cover_url: String,
    pub chapter_id: Uuid,
    pub chapter_number: i32,
    pub chapter_title: String,
    pub last_anchor_cfi: String,
    pub completion_percentage: f32,
    pub is_finished: bool,
    pub last_read_at: DateTime<Utc>,
}

// -----------------------------------------------------------------------------
// Gamification, Streaks & Badges DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReadingHeartbeatRequest {
    pub book_id: Uuid,
    #[validate(range(
        min = 1,
        max = 3600,
        message = "Seconds spent must be between 1 and 3600"
    ))]
    pub seconds_spent: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReadingHeartbeatResponse {
    pub current_streak_days: i32,
    pub longest_streak_days: i32,
    pub total_reading_seconds: i64,
    pub total_xp: i32,
    pub xp_earned: i32,
    pub streak_incremented: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BadgeDto {
    pub id: String,
    pub title_id: String,
    pub title_en: String,
    pub description_id: String,
    pub description_en: String,
    pub icon_asset: String,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserBadgeDto {
    pub id: Uuid,
    pub badge_id: String,
    pub unlocked_at: DateTime<Utc>,
    pub badge: BadgeDto,
}

// -----------------------------------------------------------------------------
// Semantic & Quote DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QuoteSearchRequest {
    #[validate(length(
        min = 3,
        max = 500,
        message = "Query must be between 3 and 500 characters"
    ))]
    pub query: String,
    #[validate(range(min = 1, max = 20, message = "Limit must be between 1 and 20"))]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QuoteSearchResultDto {
    pub chunk_id: Uuid,
    pub chapter_number: i32,
    pub chapter_title: Option<String>,
    pub cfi_range: Option<String>,
    pub content: String,
    pub similarity_score: f32,
}

// -----------------------------------------------------------------------------
// Atomic Insight Cards DTOs
// -----------------------------------------------------------------------------

/// Structured content returned from tldr_cache for recap_type = 'chapter_atomic_cards'.
/// The `cards` field is the raw JSON blob from the cache as returned by the AI pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AtomicCardsDto {
    pub book_id: Uuid,
    pub chapter_id: Uuid,
    pub cards: serde_json::Value,
}

/// Spoiler-free catch-up summary for a chapter.
/// The `recap` field is the raw JSON blob from the cache as returned by the AI pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChapterRecapDto {
    pub book_id: Uuid,
    pub chapter_id: Uuid,
    pub recap: serde_json::Value,
}

// -----------------------------------------------------------------------------
// Saved Quotes DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveQuoteRequest {
    pub book_id: Uuid,
    pub chapter_id: Uuid,
    #[validate(length(
        min = 1,
        max = 2000,
        message = "Quote text must be between 1 and 2000 characters"
    ))]
    pub quote_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SavedQuoteResponseDto {
    pub id: Uuid,
    pub book_id: Uuid,
    pub chapter_id: Uuid,
    pub quote_text: String,
    pub image_card_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// -----------------------------------------------------------------------------
// Ingestion & Admin DTOs
// -----------------------------------------------------------------------------

/// Ingestion job status snapshot, backed by the Redis `job:{id}` hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JobStatusDto {
    pub job_id: String,
    pub book_id: Uuid,
    /// One of: queued, parsing, chunking, embedding, summarizing, published, failed.
    pub status: String,
    /// Completion percentage 0-100.
    pub progress: i32,
}

/// Accepted EPUB upload response (HTTP 202).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UploadBookResponseDto {
    pub job_id: String,
    pub book_id: Uuid,
    pub status: String,
}

/// One funnel step of the chapter drop-off analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DropOffPointDto {
    pub chapter_number: i32,
    pub chapter_title: String,
    /// Distinct readers whose progress reached at least this chapter.
    pub readers_reached: i64,
    /// Percentage lost relative to chapter 1 (0.0-100.0).
    pub drop_off_pct: f32,
}
