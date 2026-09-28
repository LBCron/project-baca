//! Async HTTP client for `/api/v1/*` endpoints via `gloo-net`.
//! JWT access token persists in `localStorage` under `baca_access_token`.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::{
    ActiveProgressDto, ApiResponse, BookCatalogQuery, BookDetailDto, BookSearchQuery,
    BookSearchResultDto, BookSummaryDto, ChapterDetailDto, GuestMergeRequest, LoginRequest,
    OfflineBundleDto, QuoteSearchRequest, QuoteSearchResultDto, ReadingHeartbeatRequest,
    ReadingHeartbeatResponse, ReadingProgressUpdateDto, RefreshTokenRequest, SignupRequest,
    TokenResponse, UserProfileDto, VerifyOtpRequest,
};
use wasm_bindgen::JsCast;
use web_sys::window;

pub const API_BASE: &str = "http://localhost:8080/api/v1";
const TOKEN_KEY: &str = "baca_access_token";
const REFRESH_KEY: &str = "baca_refresh_token";

fn storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

pub fn token() -> Option<String> {
    storage()?.get_item(TOKEN_KEY).ok()?
}

pub fn set_token(token: &str) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(TOKEN_KEY, token);
    }
}

pub fn clear_token() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(TOKEN_KEY);
        let _ = storage.remove_item(REFRESH_KEY);
    }
}

pub fn set_tokens(access: &str, refresh: &str) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(TOKEN_KEY, access);
        let _ = storage.set_item(REFRESH_KEY, refresh);
    }
}

pub fn refresh_token() -> Option<String> {
    storage()?.get_item(REFRESH_KEY).ok()?
}

pub fn is_authed() -> bool {
    token().is_some()
}

fn authed(builder: gloo_net::http::RequestBuilder) -> gloo_net::http::RequestBuilder {
    match token() {
        Some(t) => builder.header("Authorization", &format!("Bearer {t}")),
        None => builder,
    }
}

async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{API_BASE}{path}");
    let envelope = authed(Request::get(&url))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ApiResponse<T>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope.data.ok_or_else(|| {
        envelope
            .error
            .map(|e| format!("{}: {}", e.code, e.message))
            .unwrap_or_else(|| "empty response".to_string())
    })
}

async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{API_BASE}{path}");
    let req = authed(Request::post(&url))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(body).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let envelope = req
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ApiResponse<T>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope.data.ok_or_else(|| {
        envelope
            .error
            .map(|e| format!("{}: {}", e.code, e.message))
            .unwrap_or_else(|| "empty response".to_string())
    })
}

async fn put<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{API_BASE}{path}");
    let req = authed(Request::put(&url))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(body).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let envelope = req
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ApiResponse<T>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope.data.ok_or_else(|| {
        envelope
            .error
            .map(|e| format!("{}: {}", e.code, e.message))
            .unwrap_or_else(|| "empty response".to_string())
    })
}
fn encode_param(value: &str) -> String {
    js_sys::encode_uri_component(value)
        .as_string()
        .unwrap_or_default()
}

pub async fn catalog(query: &BookCatalogQuery) -> Result<Vec<BookSummaryDto>, String> {
    let mut url = format!("{API_BASE}/books?limit={}", query.limit.unwrap_or(20));
    if let Some(cursor) = &query.cursor {
        url.push_str(&format!("&cursor={}", encode_param(&cursor.to_string())));
    }
    if let Some(language) = &query.language {
        url.push_str(&format!("&language={}", encode_param(language)));
    }
    if let Some(theme) = &query.theme {
        url.push_str(&format!("&theme={}", encode_param(theme)));
    }
    if let Some(tag) = &query.tag {
        url.push_str(&format!("&tag={}", encode_param(tag)));
    }
    let envelope = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ApiResponse<Vec<BookSummaryDto>>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope
        .data
        .ok_or_else(|| "empty catalog response".to_string())
}

pub async fn search(query: &BookSearchQuery) -> Result<Vec<BookSearchResultDto>, String> {
    let url = format!(
        "{API_BASE}/books/search?q={}&limit={}",
        encode_param(&query.q),
        query.limit.unwrap_or(10)
    );
    let envelope = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ApiResponse<Vec<BookSearchResultDto>>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope
        .data
        .ok_or_else(|| "empty search response".to_string())
}

pub async fn book_detail(id: &str) -> Result<BookDetailDto, String> {
    get(&format!("/books/{id}")).await
}

pub async fn chapter(book_id: &str, number: i32) -> Result<ChapterDetailDto, String> {
    get(&format!("/books/{book_id}/chapters/{number}")).await
}

pub async fn offline_bundle(book_id: &str) -> Result<OfflineBundleDto, String> {
    get(&format!("/books/{book_id}/offline-bundle")).await
}

pub async fn save_progress(book_id: &str, update: &ReadingProgressUpdateDto) -> Result<(), String> {
    let _: String = put(&format!("/progress/{book_id}"), update).await?;
    Ok(())
}

pub async fn active_progress() -> Result<Vec<ActiveProgressDto>, String> {
    get("/progress/active").await
}

pub async fn merge_guest_progress(req: &GuestMergeRequest) -> Result<(), String> {
    let _: serde_json::Value = post("/progress/merge", req).await?;
    Ok(())
}

pub async fn quote_search(
    book_id: &str,
    req: &QuoteSearchRequest,
) -> Result<Vec<QuoteSearchResultDto>, String> {
    post(&format!("/books/{book_id}/quotes/search"), req).await
}

pub async fn atomic_cards(
    book_id: &str,
    chapter_ref: &str,
) -> Result<shared::AtomicCardsDto, String> {
    get(&format!(
        "/books/{book_id}/chapters/{chapter_ref}/atomic-cards"
    ))
    .await
}

pub async fn chapter_recap(
    book_id: &str,
    chapter_ref: &str,
) -> Result<shared::ChapterRecapDto, String> {
    get(&format!("/books/{book_id}/chapters/{chapter_ref}/recap")).await
}

pub async fn signup(req: &SignupRequest) -> Result<(), String> {
    let url = format!("{API_BASE}/auth/signup");
    let request = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(req).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    request.send().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn verify_otp(req: &VerifyOtpRequest) -> Result<TokenResponse, String> {
    post("/auth/verify-otp", req).await
}

pub async fn login(req: &LoginRequest) -> Result<TokenResponse, String> {
    post("/auth/login", req).await
}

pub async fn refresh() -> Result<TokenResponse, String> {
    let rt = refresh_token().ok_or_else(|| "no refresh token".to_string())?;
    post("/auth/refresh", &RefreshTokenRequest { refresh_token: rt }).await
}

pub async fn logout() -> Result<(), String> {
    let url = format!("{API_BASE}/auth/logout");
    let rt = refresh_token().unwrap_or_default();
    let req = authed(Request::post(&url))
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&RefreshTokenRequest { refresh_token: rt })
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    req.send().await.map_err(|e| e.to_string())?;
    clear_token();
    Ok(())
}

pub async fn me() -> Result<UserProfileDto, String> {
    get("/me").await
}

pub async fn my_badges() -> Result<Vec<shared::UserBadgeDto>, String> {
    get("/me/badges").await
}

pub async fn all_badges() -> Result<Vec<shared::BadgeDto>, String> {
    get("/badges").await
}

pub async fn heartbeat(req: &ReadingHeartbeatRequest) -> Result<ReadingHeartbeatResponse, String> {
    post("/activity/heartbeat", req).await
}

pub async fn save_quote(
    req: &shared::SaveQuoteRequest,
) -> Result<shared::SavedQuoteResponseDto, String> {
    post("/quotes/save", req).await
}

pub async fn my_quotes() -> Result<Vec<shared::SavedQuoteResponseDto>, String> {
    get("/quotes").await
}

pub fn window_document() -> Option<web_sys::Document> {
    window()?.document()
}

pub fn as_html_element(node: &web_sys::Node) -> Option<web_sys::HtmlElement> {
    node.dyn_ref::<web_sys::HtmlElement>().cloned()
}
