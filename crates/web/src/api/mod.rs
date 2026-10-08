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

/// API base URL, baked at Trunk build time via `API_BASE_URL` env
/// (`Makefile` passes it through; dev falls back to localhost).
pub fn api_base() -> &'static str {
    option_env!("API_BASE_URL").unwrap_or("http://localhost:8080/api/v1")
}
const TOKEN_KEY: &str = "baca_access_token";

/// Resolves a stored asset reference to something an `<img>` or a CSS
/// `url()` can fetch: absolute URLs and root paths pass through, bucket keys
/// such as `covers/<id>.webp` are served by the API.
pub fn asset_url(reference: &str) -> Option<String> {
    let value = reference.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with('/') {
        return Some(value.to_string());
    }
    Some(format!("{}/{value}", api_base()))
}
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

use std::cell::RefCell;

thread_local! {
    static REFRESHING: RefCell<bool> = const { RefCell::new(false) };
}

async fn sleep_ms(ms: i32) {
    let Some(window) = window() else { return };
    let promise = js_sys::Promise::new(&mut move |resolve, _reject| {
        let func: &js_sys::Function = resolve.unchecked_ref();
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(func, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

use std::future::Future;
use std::pin::Pin;

type SendFuture = Pin<Box<dyn Future<Output = Result<gloo_net::http::Response, String>>>>;

/// Sends a request, transparently refreshing an expired access token once.
/// A second 401 clears the stored tokens so the next navigation lands on the
/// guest view instead of failing silently.
async fn authed_send(make: &dyn Fn() -> SendFuture) -> Result<gloo_net::http::Response, String> {
    let first = make().await?;
    if first.status() != 401 {
        return Ok(first);
    }
    if refresh_token().is_none() {
        clear_token();
        return Ok(first);
    }
    refresh_once().await?;
    let second = make().await?;
    if second.status() == 401 {
        clear_token();
    }
    Ok(second)
}

async fn parse_envelope<T: DeserializeOwned>(resp: gloo_net::http::Response) -> Result<T, String> {
    let envelope = resp
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

/// Single-flight refresh: concurrent expiries share one rotation instead of
/// racing, which would trip reuse detection and kill the whole family.
async fn refresh_once() -> Result<(), String> {
    let already = REFRESHING.with(|f| {
        let busy = *f.borrow();
        if !busy {
            *f.borrow_mut() = true;
        }
        busy
    });
    if already {
        let mut waited = 0;
        while waited < 2000 {
            sleep_ms(50).await;
            waited += 50;
            if !REFRESHING.with(|f| *f.borrow()) {
                break;
            }
        }
        return Ok(());
    }
    let result = refresh().await;
    REFRESHING.with(|f| *f.borrow_mut() = false);
    result?;
    Ok(())
}

async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
    let resp = authed_send(&|| {
        let url = url.clone();
        Box::pin(async move {
            authed(Request::get(&url))
                .send()
                .await
                .map_err(|e| e.to_string())
        }) as SendFuture
    })
    .await?;
    parse_envelope(resp).await
}

async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
    let json = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = authed_send(&|| {
        let (url, json) = (url.clone(), json.clone());
        Box::pin(async move {
            authed(Request::post(&url))
                .header("Content-Type", "application/json")
                .body(json)
                .map_err(|e| format!("{e:?}"))?
                .send()
                .await
                .map_err(|e| e.to_string())
        }) as SendFuture
    })
    .await?;
    parse_envelope(resp).await
}

/// Raw POST without the 401-refresh cycle, for the auth endpoints themselves
/// (a 401 there means bad credentials, not an expired session).
async fn post_once<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
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

async fn patch<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
    let json = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = authed_send(&|| {
        let (url, json) = (url.clone(), json.clone());
        Box::pin(async move {
            authed(Request::patch(&url))
                .header("Content-Type", "application/json")
                .body(json)
                .map_err(|e| format!("{e:?}"))?
                .send()
                .await
                .map_err(|e| e.to_string())
        }) as SendFuture
    })
    .await?;
    parse_envelope(resp).await
}

async fn put<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
    let json = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = authed_send(&|| {
        let (url, json) = (url.clone(), json.clone());
        Box::pin(async move {
            authed(Request::put(&url))
                .header("Content-Type", "application/json")
                .body(json)
                .map_err(|e| format!("{e:?}"))?
                .send()
                .await
                .map_err(|e| e.to_string())
        }) as SendFuture
    })
    .await?;
    parse_envelope(resp).await
}

fn encode_param(value: &str) -> String {
    js_sys::encode_uri_component(value)
        .as_string()
        .unwrap_or_default()
}

pub async fn catalog(query: &BookCatalogQuery) -> Result<Vec<BookSummaryDto>, String> {
    let mut url = format!("{}/books?limit={}", api_base(), query.limit.unwrap_or(20));
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
        "{}/books/search?q={}&limit={}",
        api_base(),
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

pub async fn active_progress() -> Result<Option<ActiveProgressDto>, String> {
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
    let url = format!("{}/auth/signup", api_base());
    let request = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(req).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    request.send().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn verify_otp(req: &VerifyOtpRequest) -> Result<TokenResponse, String> {
    post_once("/auth/verify-otp", req).await
}

pub async fn login(req: &LoginRequest) -> Result<TokenResponse, String> {
    post_once("/auth/login", req).await
}

pub async fn refresh() -> Result<TokenResponse, String> {
    let rt = refresh_token().ok_or_else(|| "no refresh token".to_string())?;
    let tokens: TokenResponse =
        post_once("/auth/refresh", &RefreshTokenRequest { refresh_token: rt }).await?;
    set_tokens(&tokens.access_token, &tokens.refresh_token);
    Ok(tokens)
}

pub async fn logout() -> Result<(), String> {
    let url = format!("{}/auth/logout", api_base());
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

/// Changes the password; when the server revoked other sessions it returns
/// a fresh pair, which replaces the now-dead local tokens.
pub async fn change_password(
    req: &shared::ChangePasswordRequest,
) -> Result<shared::PasswordChangedDto, String> {
    let outcome: shared::PasswordChangedDto = put("/me/password", req).await?;
    if let Some(tokens) = &outcome.tokens {
        set_tokens(&tokens.access_token, &tokens.refresh_token);
    }
    Ok(outcome)
}

pub async fn my_streak() -> Result<shared::ReadingStreakDto, String> {
    get("/me/streak").await
}

pub async fn admin_job(job_id: &str) -> Result<shared::JobStatusDto, String> {
    get(&format!("/admin/jobs/{job_id}")).await
}

pub const ADMIN_PAGE_SIZE: usize = 30;

pub async fn admin_books(
    status: Option<&str>,
    cursor: Option<uuid::Uuid>,
) -> Result<Vec<shared::AdminBookRowDto>, String> {
    let mut path = format!("/admin/books?limit={ADMIN_PAGE_SIZE}");
    if let Some(status) = status {
        path.push_str(&format!("&status={status}"));
    }
    if let Some(cursor) = cursor {
        path.push_str(&format!("&cursor={cursor}"));
    }
    get(&path).await
}

pub async fn admin_set_status(
    book_id: uuid::Uuid,
    status: &str,
) -> Result<shared::AdminBookRowDto, String> {
    patch(
        &format!("/admin/books/{book_id}"),
        &shared::AdminBookPatchRequest {
            status: status.to_string(),
        },
    )
    .await
}

pub async fn admin_dlq() -> Result<Vec<shared::DlqEntryDto>, String> {
    get("/admin/dlq?limit=100").await
}

pub async fn admin_replay(entry_id: &str) -> Result<shared::DlqReplayResponseDto, String> {
    post(
        &format!("/admin/dlq/{entry_id}/replay"),
        &serde_json::json!({}),
    )
    .await
}

pub async fn admin_dropoff(book_id: uuid::Uuid) -> Result<Vec<shared::DropOffPointDto>, String> {
    get(&format!("/admin/analytics/drop-off?book_id={book_id}")).await
}

/// Copies text to the clipboard; false when the platform refuses.
pub async fn copy_text(text: &str) -> bool {
    let Some(navigator) = window().map(|w| w.navigator()) else {
        return false;
    };
    wasm_bindgen_futures::JsFuture::from(navigator.clipboard().write_text(text))
        .await
        .is_ok()
}

/// Shares text through the Web Share API when the platform offers it.
pub async fn share_text(title: &str, text: &str) -> bool {
    let Some(navigator) = window().map(|w| w.navigator()) else {
        return false;
    };
    let data = web_sys::ShareData::new();
    data.set_title(title);
    data.set_text(text);
    wasm_bindgen_futures::JsFuture::from(navigator.share_with_data(&data))
        .await
        .is_ok()
}

pub async fn my_sessions() -> Result<Vec<shared::SessionDto>, String> {
    get("/me/sessions").await
}

pub async fn revoke_session(session_id: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/me/sessions/{session_id}", api_base());
    let resp = authed_send(&|| {
        let url = url.clone();
        Box::pin(async move {
            authed(Request::delete(&url))
                .send()
                .await
                .map_err(|e| e.to_string())
        }) as SendFuture
    })
    .await?;
    parse_envelope(resp).await
}

/// `keep_current` signs every other device out while this one stays in.
pub async fn revoke_all(keep_current: bool) -> Result<serde_json::Value, String> {
    let url = format!(
        "{}/auth/revoke-all{}",
        api_base(),
        if keep_current {
            "?keep_current=true"
        } else {
            ""
        }
    );
    let resp = authed_send(&|| {
        let url = url.clone();
        Box::pin(async move {
            authed(Request::post(&url))
                .send()
                .await
                .map_err(|e| e.to_string())
        }) as SendFuture
    })
    .await?;
    parse_envelope(resp).await
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

pub async fn save_quotes_batch(
    req: &shared::SaveQuotesBatchRequest,
) -> Result<shared::SaveQuotesBatchResponseDto, String> {
    post("/quotes/save-batch", req).await
}

/// One unsent reading-progress write, replayed FIFO when connectivity returns.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingProgress {
    pub book_id: String,
    pub update: ReadingProgressUpdateDto,
}

pub async fn queue_pending_progress(
    book_id: &str,
    update: &ReadingProgressUpdateDto,
) -> Result<(), String> {
    crate::storage::put(
        crate::storage::STORE_PENDING_SYNC,
        &PendingProgress {
            book_id: book_id.to_string(),
            update: update.clone(),
        },
    )
    .await
}

pub async fn pending_count() -> usize {
    crate::storage::get_all::<PendingProgress>(crate::storage::STORE_PENDING_SYNC)
        .await
        .map(|ops| ops.len())
        .unwrap_or(0)
}

/// Replays queued progress writes oldest-first; failures stay queued.
/// Returns the number still pending.
pub async fn drain_pending() -> usize {
    let ops: Vec<PendingProgress> = crate::storage::get_all(crate::storage::STORE_PENDING_SYNC)
        .await
        .unwrap_or_default();
    if ops.is_empty() {
        return 0;
    }
    let mut failed = Vec::new();
    for op in ops {
        if save_progress(&op.book_id, &op.update).await.is_err() {
            failed.push(op);
        }
    }
    let _ = crate::storage::clear(crate::storage::STORE_PENDING_SYNC).await;
    for op in &failed {
        let _ = queue_pending_progress(&op.book_id, &op.update).await;
    }
    failed.len()
}

pub fn window_document() -> Option<web_sys::Document> {
    window()?.document()
}

pub fn as_html_element(node: &web_sys::Node) -> Option<web_sys::HtmlElement> {
    node.dyn_ref::<web_sys::HtmlElement>().cloned()
}
