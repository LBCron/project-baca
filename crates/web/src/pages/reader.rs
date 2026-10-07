//! Paginated reader: chapter HTML flows into CSS columns one viewport wide
//! and turns page by page. The position is anchored to the first visible
//! paragraph and saved after every turn; heartbeats report real elapsed time.

use crate::api;
use crate::components::icons;
use crate::components::insights::CatchupRecap;
use crate::components::progress::ProgressBar;
use crate::components::session::use_session;
use crate::components::toast::use_toasts;
use crate::format::{reading_minutes, roman};
use crate::i18n::use_lang;
use crate::storage::{self, OfflineChapterRecord};
use crate::theme::{ReaderPrefs, ReadingTheme, FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_params_map, use_query_map};
use shared::{
    BookDetailDto, ChapterDetailDto, GuestProgressRecord, ReadingHeartbeatRequest,
    ReadingProgressUpdateDto,
};
use std::time::Duration;
use wasm_bindgen::JsCast;

const HEARTBEAT_INTERVAL_SECS: u64 = 60;
const SAVE_DEBOUNCE_MS: u64 = 1200;
const TWO_COLUMN_MIN_WIDTH: i32 = 1024;
const SWIPE_THRESHOLD_PX: i32 = 48;
/// Anchor hint meaning "open on the last page" (stepping back a chapter).
const ANCHOR_END: &str = "end";

fn side_padding(el: &web_sys::HtmlElement) -> f64 {
    web_sys::window()
        .and_then(|w| w.get_computed_style(el).ok().flatten())
        .and_then(|s| s.get_property_value("padding-left").ok())
        .and_then(|v| v.trim_end_matches("px").parse::<f64>().ok())
        .unwrap_or(24.0)
}

fn paragraphs(el: &web_sys::HtmlElement) -> Vec<web_sys::HtmlElement> {
    let Ok(nodes) = el.query_selector_all(".chapter-body p") else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|i| nodes.item(i))
        .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
        .collect()
}

/// Page index of an element from its layout offset; one page stride is the
/// viewport width because the column gap equals twice the side padding.
fn page_of(offset_left: i32, stride: i32) -> usize {
    if stride <= 0 {
        0
    } else {
        (offset_left / stride).max(0) as usize
    }
}

fn anchor_index(cfi: &str) -> Option<usize> {
    cfi.strip_prefix("p-")
        .and_then(|n| n.parse::<usize>().ok())
        .map(|n| n.saturating_sub(1))
}

async fn cached_chapter(book: &str, number: i32) -> Option<ChapterDetailDto> {
    storage::get_all::<OfflineChapterRecord>(storage::STORE_OFFLINE_CHAPTERS)
        .await
        .ok()?
        .into_iter()
        .find(|r| r.book_id.to_string() == book && r.chapter_number == number)
        .map(|r| ChapterDetailDto {
            id: r.chapter_id,
            book_id: r.book_id,
            chapter_number: r.chapter_number,
            title: r.title,
            word_count: 0,
            html_content: r.html_content,
        })
}

async fn cached_book(book: &str) -> Option<BookDetailDto> {
    storage::get_all::<BookDetailDto>(storage::STORE_OFFLINE_BOOKS)
        .await
        .ok()?
        .into_iter()
        .find(|b| b.id.to_string() == book)
}

/// Saved (chapter id, anchor) for this book from the account or the device.
async fn stored_position(book: uuid::Uuid, authed: bool) -> Option<(uuid::Uuid, String)> {
    if authed {
        api::active_progress()
            .await
            .ok()
            .flatten()
            .filter(|p| p.book_id == book)
            .map(|p| (p.chapter_id, p.last_anchor_cfi))
    } else {
        storage::get_all::<GuestProgressRecord>(storage::STORE_GUEST_PROGRESS)
            .await
            .ok()?
            .into_iter()
            .find(|r| r.book_id == book)
            .map(|r| (r.last_chapter_id, r.last_anchor_cfi))
    }
}

#[component]
fn TypeSheet(prefs: RwSignal<ReaderPrefs>) -> impl IntoView {
    let (lang, _) = use_lang();
    let adjust = move |f: &dyn Fn(&mut ReaderPrefs)| {
        prefs.update(|p| {
            f(p);
            *p = p.clamped();
        });
        prefs.get_untracked().save();
    };
    let at_smallest_font = move || prefs.get().font_size <= FONT_SIZE_MIN;
    let at_largest_font = move || prefs.get().font_size >= FONT_SIZE_MAX;
    view! {
        <section class="type-sheet" aria-label=move || lang.get().text("reader_settings")>
            <div class="sheet-grip" aria-hidden="true"></div>
            <div class="swatches" role="radiogroup" aria-label="Theme">
                {ReadingTheme::ALL.iter().map(|t| {
                    let t = *t;
                    let active = move || prefs.get().theme == t;
                    view! {
                        <button class="swatch" data-swatch=t.attr() class:active=active role="radio" aria-checked=active on:click=move |_| adjust(&|p| p.theme = t)>
                            <span class="dot" aria-hidden="true"></span>
                            {move || lang.get().text(t.label_key())}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="type-row">
                <span>{move || lang.get().text("font_size")}</span>
                <div class="stepper">
                    <button class="step-small" aria-label=move || lang.get().text("font_smaller") disabled=at_smallest_font on:click=move |_| adjust(&|p| p.font_size -= 1)>"A"</button>
                    <span class="divider" aria-hidden="true"></span>
                    <button class="step-large" aria-label=move || lang.get().text("font_larger") disabled=at_largest_font on:click=move |_| adjust(&|p| p.font_size += 1)>"A"</button>
                </div>
                <span class="visually-hidden" aria-live="polite">{move || format!("{}px", prefs.get().font_size)}</span>
            </div>
            <div class="type-row">
                <span>{move || lang.get().text("line_height")}</span>
                <div class="leading-options" role="radiogroup" aria-label=move || lang.get().text("line_height")>
                    {LEADING_PRESETS.iter().enumerate().map(|(i, (tenths, key))| {
                        let tenths = *tenths;
                        let key = *key;
                        let active = move || prefs.get().line_height == tenths;
                        view! {
                            <button class="leading-opt" class:active=active role="radio" aria-checked=active aria-label=move || lang.get().text(key) on:click=move |_| adjust(&|p| p.line_height = tenths)>
                                {icons::lines(i as u8)}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn ReaderPage() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let params = use_params_map();
    let book_id = StoredValue::new(params.get_untracked().get("id").unwrap_or_default());
    let start_chapter: i32 = use_query_map()
        .get_untracked()
        .get("chapter")
        .and_then(|c| c.parse().ok())
        .unwrap_or(1);

    let (book, set_book) = signal(None::<BookDetailDto>);
    let (chapter, set_chapter) = signal(None::<ChapterDetailDto>);
    let (chapter_no, set_chapter_no) = signal(start_chapter);
    let (error, set_error) = signal(None::<String>);
    let (offline, set_offline) = signal(false);
    let (menu_open, set_menu_open) = signal(false);
    let prefs = RwSignal::new(ReaderPrefs::load());
    let (page, set_page) = signal(0usize);
    let (pages, set_pages) = signal(1usize);
    let (column_width, set_column_width) = signal(0.0f64);
    let (anchor, set_anchor) = signal(String::from("p-1"));
    let (pending_anchor, set_pending_anchor) = signal(None::<String>);
    let (turning, set_turning) = signal(false);
    let (streak, set_streak) = signal(None::<i32>);
    let (queued, set_queued) = signal(0usize);
    let (save_generation, set_save_generation) = signal(0u32);
    let wants_recap = use_query_map().get_untracked().get("recap").is_some();
    let recap_open = RwSignal::new(wants_recap && start_chapter > 1);
    let viewport_ref = NodeRef::<leptos::html::Div>::new();
    let touch_start = StoredValue::new(None::<i32>);

    let load_chapter = move |number: i32, anchor_hint: Option<String>| {
        set_pending_anchor.set(anchor_hint);
        set_page.set(0);
        let id = book_id.get_value();
        spawn_local(async move {
            match api::chapter(&id, number).await {
                Ok(detail) => {
                    set_chapter.set(Some(detail));
                    set_error.set(None);
                    set_offline.set(false);
                }
                Err(e) => match cached_chapter(&id, number).await {
                    Some(detail) => {
                        set_chapter.set(Some(detail));
                        set_error.set(None);
                        set_offline.set(true);
                    }
                    None => set_error.set(Some(e)),
                },
            }
        });
    };
    load_chapter(start_chapter, None);

    spawn_local(async move {
        let id = book_id.get_value();
        let detail = match api::book_detail(&id).await {
            Ok(b) => Some(b),
            Err(_) => cached_book(&id).await,
        };
        let saved = match id.parse::<uuid::Uuid>() {
            Ok(uuid) => stored_position(uuid, session.authed.get_untracked()).await,
            Err(_) => None,
        };
        if let (Some(b), Some((chapter_id, cfi))) = (&detail, saved) {
            let saved_chapter = b
                .chapters
                .iter()
                .find(|c| c.id == chapter_id)
                .map(|c| c.chapter_number);
            if saved_chapter == Some(chapter_no.get_untracked()) {
                set_pending_anchor.set(Some(cfi));
            }
        }
        set_book.set(detail);
        set_queued.set(api::pending_count().await);
    });

    let measure = move || {
        let Some(el) = viewport_ref.get_untracked() else {
            return;
        };
        let width = el.client_width();
        if width <= 0 {
            return;
        }
        let padding = side_padding(&el);
        let columns = if width >= TWO_COLUMN_MIN_WIDTH {
            2.0
        } else {
            1.0
        };
        let col = (f64::from(width) - 2.0 * padding * columns) / columns;
        set_column_width.set(col.max(120.0));
    };

    let paginate = move || {
        let Some(el) = viewport_ref.get_untracked() else {
            return;
        };
        let width = el.client_width().max(1);
        let total = ((f64::from(el.scroll_width()) / f64::from(width)).ceil() as usize).max(1);
        set_pages.set(total);
        if let Some(cfi) = pending_anchor.get_untracked() {
            if cfi == ANCHOR_END {
                set_page.set(total - 1);
            } else if let Some(p) =
                anchor_index(&cfi).and_then(|i| paragraphs(&el).into_iter().nth(i))
            {
                set_page.set(page_of(p.offset_left(), width).min(total - 1));
            }
            set_pending_anchor.set(None);
        }
        set_page.update(|p| *p = (*p).min(total - 1));
    };

    Effect::new(move || {
        if chapter.get().is_none() {
            return;
        }
        let _ = prefs.get();
        request_animation_frame(move || {
            measure();
            request_animation_frame(paginate);
        });
    });

    let percent = Memo::new(move |_| {
        let Some(b) = book.get() else {
            return 0.0f32;
        };
        let total = b.chapters.len() as f32;
        if total == 0.0 {
            return 0.0;
        }
        let index = b
            .chapters
            .iter()
            .position(|c| c.chapter_number == chapter_no.get())
            .unwrap_or(0) as f32;
        let fraction = (page.get() as f32 + 1.0) / pages.get().max(1) as f32;
        (((index + fraction) / total) * 100.0).clamp(0.0, 100.0)
    });

    let persist = move || {
        let Some(ch) = chapter.get_untracked() else {
            return;
        };
        let id = book_id.get_value();
        let update = ReadingProgressUpdateDto {
            chapter_id: ch.id,
            last_anchor_cfi: anchor.get_untracked(),
            completion_percentage: percent.get_untracked(),
        };
        spawn_local(async move {
            if session.authed.get_untracked() {
                if api::save_progress(&id, &update).await.is_err() {
                    let _ = api::queue_pending_progress(&id, &update).await;
                }
                set_queued.set(api::pending_count().await);
            } else if let Ok(book_uuid) = id.parse::<uuid::Uuid>() {
                let record = GuestProgressRecord {
                    book_id: book_uuid,
                    last_chapter_id: update.chapter_id,
                    last_anchor_cfi: update.last_anchor_cfi,
                    completion_percentage: update.completion_percentage,
                    is_finished: Some(update.completion_percentage >= 100.0),
                    last_read_at: Some(chrono::Utc::now()),
                };
                let _ = storage::put(storage::STORE_GUEST_PROGRESS, &record).await;
            }
        });
    };

    let schedule_save = move || {
        let generation = save_generation.get_untracked() + 1;
        set_save_generation.set(generation);
        set_timeout(
            move || {
                if save_generation.get_untracked() == generation {
                    persist();
                }
            },
            Duration::from_millis(SAVE_DEBOUNCE_MS),
        );
    };

    Effect::new(move || {
        let current = page.get();
        let _ = pages.get();
        let Some(el) = viewport_ref.get_untracked() else {
            return;
        };
        let width = el.client_width().max(1);
        el.set_scroll_left(current as i32 * width);
        if let Some(first) = paragraphs(&el)
            .iter()
            .position(|p| page_of(p.offset_left(), width) >= current)
        {
            set_anchor.set(format!("p-{}", first + 1));
        }
        schedule_save();
    });

    let turn = move || {
        set_turning.set(true);
        set_timeout(move || set_turning.set(false), Duration::from_millis(140));
    };

    let chapter_numbers = move || {
        book.get_untracked()
            .map(|b| {
                b.chapters
                    .iter()
                    .map(|c| c.chapter_number)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    let go_chapter = move |number: i32, anchor_hint: Option<String>| {
        set_chapter_no.set(number);
        set_menu_open.set(false);
        load_chapter(number, anchor_hint);
        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
            let url = format!("/read/{}?chapter={number}", book_id.get_value());
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    };

    let next = move || {
        if page.get_untracked() + 1 < pages.get_untracked() {
            set_page.update(|p| *p += 1);
            turn();
            return;
        }
        let numbers = chapter_numbers();
        let following = numbers
            .iter()
            .position(|n| *n == chapter_no.get_untracked())
            .and_then(|i| numbers.get(i + 1))
            .copied();
        match following {
            Some(n) => go_chapter(n, None),
            None => toasts.info(lang.get_untracked().text("book_end")),
        }
    };

    let prev = move || {
        if page.get_untracked() > 0 {
            set_page.update(|p| *p -= 1);
            turn();
            return;
        }
        let numbers = chapter_numbers();
        let preceding = numbers
            .iter()
            .position(|n| *n == chapter_no.get_untracked())
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| numbers.get(i))
            .copied();
        if let Some(n) = preceding {
            go_chapter(n, Some(ANCHOR_END.to_string()));
        }
    };

    let keys = window_event_listener(leptos::ev::keydown, move |ev| match ev.key().as_str() {
        "ArrowRight" | "PageDown" => {
            ev.prevent_default();
            next();
        }
        "ArrowLeft" | "PageUp" => {
            ev.prevent_default();
            prev();
        }
        "Escape" => set_menu_open.set(false),
        _ => {}
    });
    on_cleanup(move || keys.remove());

    let resize = window_event_listener(leptos::ev::resize, move |_| {
        set_pending_anchor.set(Some(anchor.get_untracked()));
        measure();
        request_animation_frame(paginate);
    });
    on_cleanup(move || resize.remove());

    let online = window_event_listener(leptos::ev::online, move |_| {
        spawn_local(async move {
            api::drain_pending().await;
            set_queued.set(api::pending_count().await);
        });
    });
    on_cleanup(move || online.remove());

    {
        let last_tick = StoredValue::new(js_sys::Date::now());
        let handle = set_interval_with_handle(
            move || {
                let now = js_sys::Date::now();
                let elapsed = ((now - last_tick.get_value()) / 1000.0).round() as i32;
                last_tick.set_value(now);
                if !session.authed.get_untracked() || elapsed <= 0 {
                    return;
                }
                let visible = web_sys::window()
                    .and_then(|w| w.document())
                    .map(|d| d.visibility_state() == web_sys::VisibilityState::Visible)
                    .unwrap_or(true);
                let Ok(uuid) = book_id.get_value().parse::<uuid::Uuid>() else {
                    return;
                };
                if !visible {
                    return;
                }
                spawn_local(async move {
                    let request = ReadingHeartbeatRequest {
                        book_id: uuid,
                        seconds_spent: elapsed.clamp(1, 3600),
                    };
                    if let Ok(resp) = api::heartbeat(&request).await {
                        if resp.streak_incremented {
                            set_streak.set(Some(resp.current_streak_days));
                            set_timeout(move || set_streak.set(None), Duration::from_millis(4000));
                        }
                    }
                });
            },
            Duration::from_secs(HEARTBEAT_INTERVAL_SECS),
        );
        if let Ok(handle) = handle {
            on_cleanup(move || handle.clear());
        }
    }

    let minutes_left = move || {
        let words = chapter
            .get()
            .map(|c| {
                if c.word_count > 0 {
                    c.word_count
                } else {
                    (c.html_content.len() / 6) as i32
                }
            })
            .unwrap_or(0);
        let remaining = 1.0 - (page.get() as f32 + 1.0) / pages.get().max(1) as f32;
        (reading_minutes(words) as f32 * remaining).round() as i32
    };

    let toggle_menu = move |_| set_menu_open.update(|m| *m = !*m);

    view! {
        <div class="reader" data-reading=move || prefs.get().theme.attr()>
            <header class="reader-bar">
                <a href=format!("/book/{}", book_id.get_value()) class="btn-icon" aria-label=move || lang.get().text("reader_back")>{icons::back()}</a>
                <div class="reader-title">
                    <span class="kicker">{move || format!("{} {}", lang.get().text("chapter"), roman(chapter_no.get()))}</span>
                    <span class="label">{move || chapter.get().map(|c| c.title).unwrap_or_default()}</span>
                </div>
                <div class="reader-tools">
                    <button class="btn-icon btn-type" aria-label=move || lang.get().text("reader_settings") aria-expanded=move || menu_open.get() on:click=toggle_menu>"Aa"</button>
                </div>
            </header>

            <div
                class="reader-stage"
                on:touchstart=move |ev: web_sys::TouchEvent| {
                    touch_start.set_value(ev.touches().get(0).map(|t| t.client_x()));
                }
                on:touchend=move |ev: web_sys::TouchEvent| {
                    let end = ev.changed_touches().get(0).map(|t| t.client_x());
                    if let (Some(start), Some(end)) = (touch_start.get_value(), end) {
                        let dx = end - start;
                        if dx <= -SWIPE_THRESHOLD_PX {
                            next();
                        } else if dx >= SWIPE_THRESHOLD_PX {
                            prev();
                        }
                    }
                    touch_start.set_value(None);
                }
            >
                {move || (chapter_no.get() > 1).then(|| {
                    let last = chapter_no.get() - 1;
                    let range = if last == 1 { roman(1) } else { format!("I\u{2013}{}", roman(last)) };
                    view! {
                        <button class="recap-chip" on:click=move |_| recap_open.set(true)>
                            {icons::recap()}
                            <span>{lang.get().text_with("recap_range", "range", &range)}</span>
                        </button>
                    }
                })}
                <div
                    node_ref=viewport_ref
                    class="reader-viewport"
                    class:turning=move || turning.get()
                    style=move || format!(
                        "--reader-font-size: {}px; --reader-line-height: {}; column-width: {:.0}px;",
                        prefs.get().font_size,
                        prefs.get().line_height_css(),
                        column_width.get()
                    )
                >
                    {move || match chapter.get() {
                        None => match error.get() {
                            None => view! { <div class="skeleton" style="height: 60vh" aria-busy="true"></div> }.into_any(),
                            Some(_) => view! { <p class="reader-end">{move || lang.get().text("not_found")}</p> }.into_any(),
                        },
                        Some(c) => view! { <div class="chapter-body" inner_html=c.html_content></div> }.into_any(),
                    }}
                </div>
                <button class="reader-zone reader-zone-prev" aria-label=move || lang.get().text("prev_page") on:click=move |_| prev()></button>
                <button class="reader-zone reader-zone-menu" aria-label=move || lang.get().text("toggle_menu") on:click=toggle_menu></button>
                <button class="reader-zone reader-zone-next" aria-label=move || lang.get().text("next_page") on:click=move |_| next()></button>
                {move || menu_open.get().then(|| view! { <TypeSheet prefs=prefs/> })}
            </div>

            <footer class="reader-footer">
                <ProgressBar percent=percent label=Signal::derive(move || lang.get().text("reading_progress")) thin=true/>
                <div class="reader-status">
                    <span>{move || lang.get().text_with("page_of", "a", &(page.get() + 1).to_string()).replace("{b}", &pages.get().to_string())}</span>
                    <span>
                        {move || lang.get().text_with("minutes_left", "n", &minutes_left().to_string())}
                        {move || offline.get().then(|| view! { <span class="badge">{format!(" \u{00B7} {}", lang.get().text("offline_copy"))}</span> })}
                        {move || (queued.get() > 0).then(|| view! { <span class="badge">{format!(" \u{00B7} {}", lang.get().text_with("queued", "n", &queued.get().to_string()))}</span> })}
                    </span>
                </div>
            </footer>

            {move || view! { <CatchupRecap book_id=book_id.get_value() chapter=chapter_no.get() show=recap_open/> }}
            {move || streak.get().map(|days| view! {
                <div class="streak-toast" role="status">{format!("\u{2756} {}", lang.get().text_with("streak_days", "n", &days.to_string()))}</div>
            })}
        </div>
    }
}
