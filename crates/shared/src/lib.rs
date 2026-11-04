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

/// The `error.code` values the API answers with. The server sends only
/// these, and the web client keeps one dictionary entry for each.
pub mod error_code {
    pub const BAD_REQUEST: &str = "BAD_REQUEST";
    pub const VALIDATION_FAILED: &str = "VALIDATION_FAILED";
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const FORBIDDEN: &str = "FORBIDDEN";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const METHOD_NOT_ALLOWED: &str = "METHOD_NOT_ALLOWED";
    pub const CONFLICT: &str = "CONFLICT";
    pub const QUOTA_EXCEEDED: &str = "QUOTA_EXCEEDED";
    pub const PAYLOAD_TOO_LARGE: &str = "PAYLOAD_TOO_LARGE";
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    pub const AI_RATE_LIMITED: &str = "AI_RATE_LIMITED";
    pub const SERVICE_UNAVAILABLE: &str = "SERVICE_UNAVAILABLE";
    pub const EXTERNAL_SERVICE_ERROR: &str = "EXTERNAL_SERVICE_ERROR";
    pub const DATABASE_ERROR: &str = "DATABASE_ERROR";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";

    pub const ALL: &[&str] = &[
        BAD_REQUEST,
        VALIDATION_FAILED,
        UNAUTHORIZED,
        FORBIDDEN,
        NOT_FOUND,
        METHOD_NOT_ALLOWED,
        CONFLICT,
        QUOTA_EXCEEDED,
        PAYLOAD_TOO_LARGE,
        RATE_LIMITED,
        AI_RATE_LIMITED,
        SERVICE_UNAVAILABLE,
        EXTERNAL_SERVICE_ERROR,
        DATABASE_ERROR,
        INTERNAL_ERROR,
    ];
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

    /// A per-account storage limit is spent; distinct from `Conflict` so the
    /// client can say what the reader can do about it.
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    /// The body is larger than the route accepts (413).
    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Service temporarily unavailable. Try again in {retry_after} seconds")]
    ServiceUnavailable { retry_after: u64 },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

// Authentication & User DTOs

/// Canonical form of an address: one account and one lockout key per
/// mailbox, whatever case or padding the client sent.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn deserialize_email<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    String::deserialize(deserializer).map(|email| normalize_email(&email))
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SignupRequest {
    #[serde(alias = "nickname")]
    #[validate(length(
        min = 2,
        max = 100,
        message = "Display name must be between 2 and 100 characters"
    ))]
    #[cfg_attr(feature = "openapi", schema(min_length = 2, max_length = 100))]
    pub display_name: String,

    #[serde(deserialize_with = "deserialize_email")]
    #[validate(email(message = "Invalid email format"))]
    #[cfg_attr(feature = "openapi", schema(format = Email))]
    pub email: String,

    #[validate(
        length(min = 8, max = 128, message = "Password must be at least 8 characters"),
        custom(function = "validate_not_common_password")
    )]
    #[cfg_attr(feature = "openapi", schema(min_length = 8, max_length = 128))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VerifyOtpRequest {
    #[serde(deserialize_with = "deserialize_email")]
    #[validate(email(message = "Invalid email format"))]
    #[cfg_attr(feature = "openapi", schema(format = Email))]
    pub email: String,

    /// The registration the code was mailed for, from the signup answer.
    pub registration_id: Uuid,

    #[validate(length(equal = 6, message = "OTP must be exactly 6 digits"))]
    #[cfg_attr(feature = "openapi", schema(min_length = 6, max_length = 6))]
    pub otp: String,
}

/// Answer to a signup: the code is on its way and must come back with this
/// registration's id.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegistrationStartedDto {
    pub message: String,
    pub email: String,
    pub registration_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LoginRequest {
    #[serde(deserialize_with = "deserialize_email")]
    #[validate(email(message = "Invalid email format"))]
    #[cfg_attr(feature = "openapi", schema(format = Email))]
    pub email: String,

    #[validate(length(min = 1, message = "Password cannot be empty"))]
    #[cfg_attr(feature = "openapi", schema(min_length = 1))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token must not be empty"))]
    #[cfg_attr(feature = "openapi", schema(min_length = 1))]
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
    #[cfg_attr(feature = "openapi", schema(min_length = 2, max_length = 100))]
    pub display_name: Option<String>,
    #[validate(custom(function = "validate_avatar_url"))]
    pub avatar_url: Option<String>,
}

/// Avatar URLs are rendered by clients, never fetched server-side. Only
/// `http(s)://` is accepted so `javascript:`/`data:` payloads cannot be
/// stored for later XSS.
pub fn validate_avatar_url(url: &str) -> Result<(), validator::ValidationError> {
    if url.len() > 2048 {
        let mut err = validator::ValidationError::new("avatar_url_too_long");
        err.message = Some("Avatar URL must not exceed 2048 characters".into());
        return Err(err);
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        let mut err = validator::ValidationError::new("avatar_url_scheme");
        err.message = Some("Avatar URL must start with https:// or http://".into());
        return Err(err);
    }
    Ok(())
}

/// The SecLists 10k most common passwords (MIT), lowercased and sorted, kept
/// to those of 8 characters or more: shorter ones already fail the length rule.
const COMMON_PASSWORDS: &str = include_str!("common_passwords.txt");

/// Validator code of a password found in the common list; clients choose
/// their copy by it.
pub const COMMON_PASSWORD: &str = "common_password";

/// Refuses a password from the common list, whatever its case.
pub fn validate_not_common_password(password: &str) -> Result<(), validator::ValidationError> {
    let lowered = password.to_lowercase();
    if COMMON_PASSWORDS.lines().any(|common| common == lowered) {
        let mut err = validator::ValidationError::new(COMMON_PASSWORD);
        err.message = Some("Password is too common".into());
        return Err(err);
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    #[cfg_attr(feature = "openapi", schema(min_length = 1))]
    pub current_password: String,
    #[validate(
        length(
            min = 8,
            max = 128,
            message = "New password must be at least 8 characters"
        ),
        custom(function = "validate_not_common_password")
    )]
    #[cfg_attr(feature = "openapi", schema(min_length = 8, max_length = 128))]
    pub new_password: String,
    #[serde(default)]
    pub revoke_other_sessions: Option<bool>,
}

/// Outcome of a password change. `tokens` is present when other sessions were
/// revoked, so the caller can replace its own now-invalid pair without a
/// second login.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PasswordChangedDto {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GuestProgressRecord {
    pub book_id: Uuid,
    pub last_chapter_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    #[cfg_attr(feature = "openapi", schema(min_length = 1, max_length = 255))]
    pub last_anchor_cfi: String,
    #[validate(range(min = 0.0, max = 100.0))]
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0))]
    pub completion_percentage: f32,
    pub is_finished: Option<bool>,
    pub last_read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GuestMergeRequest {
    #[validate(length(max = 100, message = "Cannot merge more than 100 records at once"))]
    #[validate(nested)]
    #[cfg_attr(feature = "openapi", schema(max_items = 100))]
    pub records: Vec<GuestProgressRecord>,
}

// Catalog & Book DTOs

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

/// Filter bounds match the column widths they are compared against
/// (`language` VARCHAR(10), `primary_theme` and `tags.name` VARCHAR(50)).
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct BookCatalogQuery {
    pub cursor: Option<Uuid>,
    /// Page size, clamped to 1..=100 (default 20).
    pub limit: Option<i64>,
    #[validate(length(max = 10, message = "Language filter must not exceed 10 characters"))]
    #[cfg_attr(feature = "openapi", param(max_length = 10))]
    pub language: Option<String>,
    #[validate(length(max = 50, message = "Theme filter must not exceed 50 characters"))]
    #[cfg_attr(feature = "openapi", param(max_length = 50))]
    pub theme: Option<String>,
    #[validate(length(max = 50, message = "Tag filter must not exceed 50 characters"))]
    #[cfg_attr(feature = "openapi", param(max_length = 50))]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct BookSearchQuery {
    #[validate(length(
        min = 2,
        max = 200,
        message = "Search query must be between 2 and 200 characters"
    ))]
    #[cfg_attr(feature = "openapi", param(min_length = 2, max_length = 200))]
    pub q: String,
    #[validate(length(max = 10, message = "Language filter must not exceed 10 characters"))]
    #[cfg_attr(feature = "openapi", param(max_length = 10))]
    pub language: Option<String>,
    /// Result count, clamped to 1..=50 (default 20).
    pub limit: Option<i64>,
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

/// `GET /books/{id}/search`: a phrase looked up in one book's text.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct BookTextSearchQuery {
    #[validate(length(
        min = 2,
        max = 200,
        message = "Search query must be between 2 and 200 characters"
    ))]
    #[cfg_attr(feature = "openapi", param(min_length = 2, max_length = 200))]
    pub q: String,
}

/// One paragraph of a book that contains the searched phrase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BookTextHitDto {
    pub chapter_number: i32,
    pub chapter_title: String,
    /// 1-based paragraph index in the chapter: the reader's `p-N` anchor.
    pub paragraph: i32,
    pub snippet: String,
}

/// Most hits one in-book search returns.
pub const BOOK_TEXT_HITS_MAX: usize = 50;

/// Characters of context kept on each side of a match in a snippet.
const SNIPPET_CONTEXT: usize = 70;

/// Case folding for matching, one character at a time on both sides: a
/// whole-string lowercase turns a final capital sigma into `ς` and `İ` into
/// two characters, so a needle and the text it came from stopped matching.
pub fn fold(c: char) -> char {
    match c.to_lowercase().next().unwrap_or(c) {
        '\u{03C2}' => '\u{03C3}',
        lower => lower,
    }
}

/// A query as in-book search matches it: whitespace collapsed, then folded
/// like the `chapter_paragraphs.folded` column the worker writes.
pub fn fold_for_search(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(fold)
        .collect()
}

/// The match of `query` in `text` (case-insensitive) with some context on
/// either side, or `None` when it is absent. Server and offline search share
/// it, so a hit reads the same either way.
pub fn text_hit_snippet(text: &str, query: &str) -> Option<String> {
    let needle: Vec<char> = fold_for_search(query).chars().collect();
    if needle.is_empty() {
        return None;
    }
    let words: Vec<char> = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .collect();
    let folded: Vec<char> = words.iter().map(|c| fold(*c)).collect();
    let at = folded
        .windows(needle.len())
        .position(|window| window == needle.as_slice())?;
    let start = at.saturating_sub(SNIPPET_CONTEXT);
    let end = (at + needle.len() + SNIPPET_CONTEXT).min(words.len());
    let body: String = words[start..end].iter().collect();
    Some(format!(
        "{}{}{}",
        if start > 0 { "\u{2026}" } else { "" },
        body.trim(),
        if end < words.len() { "\u{2026}" } else { "" }
    ))
}

// Reading Progress & CFI DTOs

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
    #[cfg_attr(feature = "openapi", schema(min_length = 1, max_length = 255))]
    pub last_anchor_cfi: String,
    #[serde(alias = "percentage")]
    #[validate(range(
        min = 0.0,
        max = 100.0,
        message = "Percentage must be between 0.0 and 100.0"
    ))]
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0))]
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

// Gamification, Streaks & Badges DTOs

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReadingHeartbeatRequest {
    pub book_id: Uuid,
    #[validate(range(
        min = 1,
        max = 3600,
        message = "Seconds spent must be between 1 and 3600"
    ))]
    #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 3600))]
    pub seconds_spent: i32,
    /// Minutes the reader's clock is east of UTC (WIB 420, WIT 540): the
    /// day the reading counts for is theirs. Absent means UTC.
    #[serde(default)]
    #[validate(range(min = -720, max = 840, message = "UTC offset must be between -720 and 840 minutes"))]
    #[cfg_attr(feature = "openapi", schema(minimum = -720, maximum = 840))]
    pub utc_offset_minutes: Option<i32>,
}

/// `GET /me/streak`: today's reading is summed over the reader's own day.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct StreakQuery {
    #[validate(range(min = -720, max = 840, message = "UTC offset must be between -720 and 840 minutes"))]
    #[cfg_attr(feature = "openapi", param(minimum = -720, maximum = 840))]
    pub utc_offset_minutes: Option<i32>,
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
    /// False when the heartbeat came inside the per-reader interval (two
    /// open tabs): nothing was logged and the standing is returned as is.
    /// Absent from a server older than the field, where every 200 credited.
    #[serde(default = "credited_by_default")]
    pub credited: bool,
}

fn credited_by_default() -> bool {
    true
}

/// Current streak standing for the profile page; zeros before the first
/// heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReadingStreakDto {
    pub current_streak_days: i32,
    pub longest_streak_days: i32,
    pub total_reading_seconds: i64,
    pub total_xp: i32,
    pub last_activity_date: Option<chrono::NaiveDate>,
    /// Seconds read today toward the daily threshold.
    pub today_seconds: i64,
    pub daily_threshold_seconds: i64,
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

// Semantic & Quote DTOs

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QuoteSearchRequest {
    #[validate(length(
        min = 3,
        max = 500,
        message = "Query must be between 3 and 500 characters"
    ))]
    #[cfg_attr(feature = "openapi", schema(min_length = 3, max_length = 500))]
    pub query: String,
    #[validate(range(min = 1, max = 20, message = "Limit must be between 1 and 20"))]
    #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 20))]
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

// Atomic Insight Cards DTOs

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

// Saved Quotes DTOs

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
    #[cfg_attr(feature = "openapi", schema(min_length = 1, max_length = 2000))]
    pub quote_text: String,
}

/// One signed-in device, from the refresh token it holds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SessionDto {
    /// Opaque session id (hash of the refresh token), used to revoke it.
    pub id: String,
    /// Browser and platform, empty when unknown.
    pub device: String,
    /// Coarse network location, never the full address.
    pub ip_prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    /// True for the session that made this request.
    pub current: bool,
}

/// Upper bound of one batch save, matching the guest merge chunk size.
pub const SAVE_QUOTES_BATCH_MAX: usize = 50;

/// Saved quotes listed per page when the client names no limit.
pub const SAVED_QUOTES_PAGE: u64 = 50;

/// One page of saved quotes, newest first. `cursor` is the id of the last
/// quote of the previous page; a page shorter than `limit` is the last.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct SavedQuotesQuery {
    pub cursor: Option<Uuid>,
    #[validate(range(min = 1, max = 100, message = "limit must be between 1 and 100"))]
    #[cfg_attr(feature = "openapi", param(minimum = 1, maximum = 100))]
    pub limit: Option<u64>,
}

/// Several quotes saved together, used to merge a device's local quotes into
/// the account at sign-in. Each item is validated like a single save.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveQuotesBatchRequest {
    #[validate(
        length(min = 1, max = 50, message = "A batch holds between 1 and 50 quotes"),
        nested
    )]
    #[cfg_attr(feature = "openapi", schema(min_items = 1, max_items = 50))]
    pub items: Vec<SaveQuoteRequest>,
}

/// Result for one item of a batch save, in request order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveQuoteOutcomeDto {
    pub index: usize,
    /// One of: saved, duplicate, rejected.
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveQuotesBatchResponseDto {
    pub saved: usize,
    pub duplicates: usize,
    pub rejected: usize,
    pub outcomes: Vec<SaveQuoteOutcomeDto>,
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

// Ingestion & Admin DTOs

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
    /// Why the job failed, or why its last attempt did while it waits for
    /// a retry, in the worker's words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Runs the worker has started for this job, crashes included.
    #[serde(default)]
    pub attempts: i32,
}

/// Largest EPUB a curator may upload; the server enforces it while
/// streaming and the web checks it before sending.
pub const MAX_EPUB_BYTES: usize = 50 * 1024 * 1024;

/// Accepted EPUB upload response (HTTP 202).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UploadBookResponseDto {
    pub job_id: String,
    pub book_id: Uuid,
    pub status: String,
}

/// One row of the curator's catalog table: every status, with content counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminBookRowDto {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub language: String,
    /// One of: draft, processing, published, archived.
    pub status: String,
    pub chapter_count: i64,
    pub chunk_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status change requested by a curator; the domain lifecycle decides.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminBookPatchRequest {
    #[validate(length(min = 1, max = 20))]
    #[cfg_attr(feature = "openapi", schema(min_length = 1, max_length = 20))]
    pub status: String,
}

/// One dead-letter entry of the ingestion queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DlqEntryDto {
    /// Redis stream entry id, used to replay.
    pub id: String,
    pub job_id: String,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_id: Option<Uuid>,
    /// Title of the book, while its row exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub book_title: Option<String>,
    pub failed_at: DateTime<Utc>,
}

/// One page of dead letters, newest first, and how many the stream holds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DlqPageDto {
    pub total: u64,
    pub entries: Vec<DlqEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DlqReplayResponseDto {
    pub job_id: String,
    pub replayed: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_finds_the_phrase_case_insensitively_with_context() {
        let text = "Pelabuhan Teluk bangun lebih pagi daripada penghuninya.";
        assert_eq!(
            text_hit_snippet(text, "teluk BANGUN").as_deref(),
            Some("Pelabuhan Teluk bangun lebih pagi daripada penghuninya.")
        );
        assert_eq!(text_hit_snippet(text, "kapal"), None);
        assert_eq!(text_hit_snippet(text, "  "), None);
    }

    #[test]
    fn snippet_trims_long_paragraphs_around_the_match() {
        let text = format!("{} harbour {}", "a".repeat(300), "b".repeat(300));
        let snippet = text_hit_snippet(&text, "harbour").unwrap();
        assert!(snippet.starts_with('\u{2026}') && snippet.ends_with('\u{2026}'));
        assert!(snippet.contains("harbour"));
        assert!(snippet.chars().count() <= 2 * 70 + "harbour".len() + 2);
    }

    #[test]
    fn snippet_folds_both_sides_alike() {
        assert!(text_hit_snippet("\u{0130}STANBUL by the sea", "istanbul").is_some());
        assert!(text_hit_snippet(
            "THE WANDERINGS OF \u{039F}\u{0394}\u{03A5}\u{03A3}\u{03A3}\u{0395}\u{03A5}\u{03A3}",
            "\u{03BF}\u{03B4}\u{03C5}\u{03C3}\u{03C3}\u{03B5}\u{03C5}\u{03C2}"
        )
        .is_some());
        assert!(text_hit_snippet("the secure ancestral hall", "secure   ancestral").is_some());
        assert!(text_hit_snippet("Tom & Jerry", "tom & jerry").is_some());
    }

    #[test]
    fn folding_matches_the_worker() {
        // The same literal is asserted for the worker's fold_for_search.
        assert_eq!(
            fold_for_search("\u{0130}STANBUL \u{039F}\u{0394}\u{03A5}\u{03A3}\u{03A3}\u{0395}\u{03A5}\u{03A3} \u{03C2}"),
            "istanbul \u{03BF}\u{03B4}\u{03C5}\u{03C3}\u{03C3}\u{03B5}\u{03C5}\u{03C3} \u{03C3}"
        );
    }

    #[test]
    fn snippet_collapses_markup_whitespace() {
        assert_eq!(
            text_hit_snippet("  satu \n\n  dua   tiga ", "dua").as_deref(),
            Some("satu dua tiga")
        );
    }

    #[test]
    fn common_passwords_are_refused_in_any_case() {
        for password in ["password", "12345678", "qwerty123", "PassWord", "QWERTY123"] {
            let err = validate_not_common_password(password).unwrap_err();
            assert_eq!(err.code, COMMON_PASSWORD, "{password}");
        }
        assert!(validate_not_common_password("pelabuhan teluk senja").is_ok());
    }

    #[test]
    fn signup_and_password_change_name_the_common_password_code() {
        let signup = SignupRequest {
            display_name: "Pembaca".into(),
            email: "pembaca@example.com".into(),
            password: "Password".into(),
        };
        let errors = signup.validate().unwrap_err();
        assert_eq!(errors.field_errors()["password"][0].code, COMMON_PASSWORD);
        let change = ChangePasswordRequest {
            current_password: "lama sekali".into(),
            new_password: "qwerty123".into(),
            revoke_other_sessions: None,
        };
        let errors = change.validate().unwrap_err();
        assert_eq!(
            errors.field_errors()["new_password"][0].code,
            COMMON_PASSWORD
        );
    }

    #[test]
    fn the_common_list_is_lowercase_sorted_and_long_enough() {
        let list: Vec<&str> = COMMON_PASSWORDS.lines().collect();
        assert!(list.len() > 2000);
        assert!(list.windows(2).all(|w| w[0] < w[1]));
        assert!(list.iter().all(|p| p.len() >= 8 && *p == p.to_lowercase()));
    }
}
