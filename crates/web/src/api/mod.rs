//! Async HTTP client for `/api/v1/*` endpoints via `gloo-net`.
//! JWT access token persists in `localStorage` under `baca_access_token`.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::{
    ActiveProgressDto, ApiResponse, BookCatalogQuery, BookDetailDto, BookSearchQuery,
    BookSearchResultDto, BookSummaryDto, ChapterDetailDto, GuestMergeRequest, LoginRequest,
    OfflineBundleDto, QuoteSearchRequest, QuoteSearchResultDto, ReadingProgressUpdateDto,
    SignupRequest, VerifyOtpRequest,
};
use wasm_bindgen::JsCast;
use web_sys::window;

const API_BASE: &str = "http://localhost:8080/api/v1";
const TOKEN_KEY: &str = "baca_access_token";

fn storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

fn token() -> Option<String> {
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
    }
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

pub async fn catalog(query: &BookCatalogQuery) -> Result<Vec<BookSummaryDto>, String> {
    let mut url = format!("{API_BASE}/books?limit={}", query.limit.unwrap_or(20));
    if let Some(cursor) = &query.cursor {
        url.push_str(&format!("&cursor={cursor}"));
    }
    if let Some(language) = &query.language {
        url.push_str(&format!("&language={language}"));
    }
    if let Some(theme) = &query.theme {
        url.push_str(&format!("&theme={theme}"));
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
        query.q,
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
    let url = format!("{API_BASE}/progress/{book_id}");
    let req = authed(Request::post(&url))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(update).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    req.send().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn active_progress() -> Result<Vec<ActiveProgressDto>, String> {
    get("/progress/active").await
}

pub async fn merge_guest_progress(req: &GuestMergeRequest) -> Result<(), String> {
    let url = format!("{API_BASE}/progress/merge");
    let request = authed(Request::post(&url))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(req).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    request.send().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn quote_search(
    book_id: &str,
    req: &QuoteSearchRequest,
) -> Result<Vec<QuoteSearchResultDto>, String> {
    post(&format!("/books/{book_id}/quotes/search"), req).await
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

pub async fn verify_otp(req: &VerifyOtpRequest) -> Result<serde_json::Value, String> {
    post("/auth/verify-otp", req).await
}

pub async fn login(req: &LoginRequest) -> Result<serde_json::Value, String> {
    post("/auth/login", req).await
}

pub fn window_document() -> Option<web_sys::Document> {
    window()?.document()
}

pub fn as_html_element(node: &web_sys::Node) -> Option<web_sys::HtmlElement> {
    node.dyn_ref::<web_sys::HtmlElement>().cloned()
}
