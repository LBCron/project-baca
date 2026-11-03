//! Paginated reader laid out as an open book: chapter HTML flows into CSS
//! columns one viewport wide (two columns on wide screens), pages turn like
//! paper, the position is anchored to the first visible paragraph and saved
//! after every turn, and heartbeats report real elapsed time.

mod flip;
mod layout;
mod tools;

use crate::api;
use crate::components::anchor::prefers_reduced_motion;
use crate::components::focus::close_after_event;
use crate::components::icons;
use crate::components::insights::{keep_quote, AtomicCards, CatchupRecap};
use crate::components::progress::ProgressBar;
use crate::components::session::use_session;
use crate::components::toast::{use_toasts, ToastStack};
use crate::format::{reading_minutes, roman};
use crate::i18n::use_lang;
use crate::storage::{self, OfflineChapterRecord};
use crate::theme::{ReaderPrefs, ReadingTheme, FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS};
use flip::{Dir, DragStart, Flipper, Geometry, Mode, TurnPlan};
use layout::{
    anchor_index, columns_for, paddings, page_of, page_of_spot, paragraphs, spot_on, watch_size,
    window_width, Spot,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_params_map, use_query_map};
use shared::{
    BookDetailDto, ChapterDetailDto, GuestProgressRecord, ReadingHeartbeatRequest,
    ReadingProgressUpdateDto,
};
use std::time::Duration;
use tools::{clear_selection, selected_chapter_text, watch_selection, BookSearch, ChapterDrawer};
use wasm_bindgen::JsCast;

const HEARTBEAT_INTERVAL_SECS: u64 = 60;
const SAVE_DEBOUNCE_MS: u64 = 1200;
/// Pointer travel below this is a tap; beyond it, a drag.
const TAP_PX: f64 = 6.0;
/// Travel that turns a chapter boundary on release.
const CHAPTER_SWIPE_PX: f64 = 48.0;
/// Outer share of the page that acts as a tap zone on either side.
const EDGE_ZONE: f64 = 0.22;
/// Anchor hint meaning "open on the last page" (stepping back a chapter).
const ANCHOR_END: &str = "end";
/// Anchor hint meaning "return to the spot being read" after a relayout.
const ANCHOR_SPOT: &str = "spot";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
struct Gesture {
    x0: f64,
    y0: f64,
    zone: Zone,
    touch: bool,
    dragging: bool,
    book_left: f64,
    book_right: f64,
    crossing: Option<i32>,
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

/// Buttons and links inside the stage handle their own clicks; a gesture
/// that begins on one must not also count as a tap on the page.
fn starts_on_control(ev: &web_sys::PointerEvent) -> bool {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .and_then(|el| el.closest("button, a, .type-sheet").ok().flatten())
        .is_some()
}

/// Keys typed into a field belong to the field; Space on a button or link
/// activates it instead of turning the page.
fn key_belongs_to_target(ev: &web_sys::KeyboardEvent) -> bool {
    let Some(el) = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    else {
        return false;
    };
    let within = |selector: &str| el.closest(selector).ok().flatten().is_some();
    within("input, select, textarea, [contenteditable]") || (ev.key() == " " && within("button, a"))
}

fn selection_is_collapsed() -> bool {
    web_sys::window()
        .and_then(|w| w.get_selection().ok().flatten())
        .map(|s| s.is_collapsed())
        .unwrap_or(true)
}

/// Copy key that explains why a chapter did not load.
fn load_failure(err: &str) -> &'static str {
    if api::is_not_found(err) {
        "not_found"
    } else if api::is_online() {
        "chapter_load_failed"
    } else {
        "offline_notice"
    }
}

/// Stores a guest's position on this device.
fn persist_guest(book: &str, update: &ReadingProgressUpdateDto) {
    let Ok(book_id) = book.parse::<uuid::Uuid>() else {
        return;
    };
    let record = GuestProgressRecord {
        book_id,
        last_chapter_id: update.chapter_id,
        last_anchor_cfi: update.last_anchor_cfi.clone(),
        completion_percentage: update.completion_percentage,
        is_finished: Some(update.completion_percentage >= 100.0),
        last_read_at: Some(chrono::Utc::now()),
    };
    spawn_local(async move {
        let _ = storage::put(storage::STORE_GUEST_PROGRESS, &record).await;
    });
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
            <div class="swatches" role="radiogroup" aria-label=move || lang.get().text("reading_theme")>
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
    crate::i18n::use_page_title(move |_| {
        let book = book.get()?.title;
        Some(match chapter.get() {
            Some(c) if !c.title.trim().is_empty() => format!("{} \u{00B7} {book}", c.title),
            _ => book,
        })
    });
    let (chapter_no, set_chapter_no) = signal(start_chapter);
    let (error, set_error) = signal(None::<String>);
    let (offline, set_offline) = signal(false);
    let (menu_open, set_menu_open) = signal(false);
    let prefs = RwSignal::new(ReaderPrefs::load());
    let (page, set_page) = signal(0usize);
    let (pages, set_pages) = signal(1usize);
    let (spread, set_spread) = signal(false);
    let (dim, set_dim) = signal(false);
    let (finished, set_finished) = signal(false);
    let (column_width, set_column_width) = signal(0.0f64);
    let (anchor, set_anchor) = signal(String::from("p-1"));
    let (pending_anchor, set_pending_anchor) = signal(None::<String>);
    let spot = StoredValue::new(None::<Spot>);
    let (turning, set_turning) = signal(false);
    let (streak, set_streak) = signal(None::<i32>);
    let (queued, set_queued) = signal(0usize);
    let (save_generation, set_save_generation) = signal(0u32);
    // Until the saved position is known, page one is not a position worth
    // saving: it would overwrite the real one when the chapter loads first.
    let (position_known, set_position_known) = signal(false);
    let save_pending = StoredValue::new(false);
    let load_generation = StoredValue::new(0u32);
    let drawer_open = RwSignal::new(false);
    let search_open = RwSignal::new(false);
    let cards_open = RwSignal::new(false);
    let (chapter_end, set_chapter_end) = signal(false);
    // Chapters whose closing card was shown on this visit: once each.
    let ends_shown = StoredValue::new(std::collections::HashSet::<i32>::new());
    let (selection, set_selection) = signal(None::<String>);
    // (chapter, paragraph) a search hit opened, highlighted once it lands.
    let (flash, set_flash) = signal(None::<(i32, usize)>);
    let wants_recap = use_query_map().get_untracked().get("recap").is_some();
    // A quote search hit opens on its paragraph, lit like an in-book hit.
    let start_anchor = use_query_map()
        .get_untracked()
        .get("at")
        .filter(|a| anchor_index(a).is_some());
    if let Some(index) = start_anchor.as_deref().and_then(anchor_index) {
        set_flash.set(Some((start_chapter, index)));
        // Read once: a reload afterwards returns to the saved place, not the hit.
        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
            let url = format!("/read/{}?chapter={start_chapter}", book_id.get_value());
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    let recap_open = RwSignal::new(wants_recap && start_chapter > 1);
    let viewport_ref = NodeRef::<leptos::html::Div>::new();
    let book_ref = NodeRef::<leptos::html::Div>::new();
    let flip_ref = NodeRef::<leptos::html::Div>::new();
    let stage_ref = NodeRef::<leptos::html::Div>::new();
    let flipper = StoredValue::new_local(None::<Flipper>);
    let gesture = StoredValue::new(None::<Gesture>);

    Effect::new(move || {
        if flipper.with_value(|f| f.is_some()) {
            return;
        }
        if let (Some(layer), Some(viewport)) = (flip_ref.get(), viewport_ref.get()) {
            let layer: web_sys::HtmlElement = (*layer).clone();
            let viewport: web_sys::HtmlElement = (*viewport).clone();
            flipper.set_value(Some(Flipper::new(layer, viewport, move |target| {
                set_page.set(target);
            })));
        }
    });
    let cancel_flip = move || {
        flipper.with_value(|f| {
            if let Some(f) = f {
                f.cancel();
            }
        });
    };
    let flip_active = move || flipper.with_value(|f| f.as_ref().is_some_and(|f| f.is_active()));

    let load_chapter = move |number: i32, anchor_hint: Option<String>| {
        cancel_flip();
        let id = book_id.get_value();
        let generation = load_generation.get_value() + 1;
        load_generation.set_value(generation);
        spawn_local(async move {
            let loaded = match api::chapter(&id, number).await {
                Ok(detail) => Ok((detail, false)),
                Err(e) => cached_chapter(&id, number)
                    .await
                    .map(|d| (d, true))
                    .ok_or(e),
            };
            // A slower answer for a chapter the reader already left must
            // neither replace the current text nor be saved under its number.
            if load_generation.try_get_value() != Some(generation) {
                return;
            }
            match loaded {
                // The reader moves only onto text it can show: the chapter
                // number, the URL and the position to save follow the text.
                Ok((detail, from_device)) => {
                    let moved = number != chapter_no.get_untracked();
                    if moved {
                        set_chapter_no.set(number);
                        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                            let url = format!("/read/{id}?chapter={number}");
                            let _ = history.replace_state_with_url(
                                &wasm_bindgen::JsValue::NULL,
                                "",
                                Some(&url),
                            );
                        }
                    }
                    spot.set_value(None);
                    // On the chapter already open (first load, retry), a saved
                    // position that answered first stays pending.
                    if moved || anchor_hint.is_some() {
                        set_pending_anchor.set(anchor_hint);
                    }
                    set_page.set(0);
                    set_finished.set(false);
                    set_chapter_end.set(false);
                    set_chapter.set(Some(detail));
                    set_error.set(None);
                    set_offline.set(from_device);
                }
                // A chapter already on screen stays there; the error view
                // takes over only while none has loaded yet.
                Err(e) => {
                    if chapter.with_untracked(Option::is_some) {
                        toasts.error(lang.get_untracked().text(load_failure(&e)));
                    }
                    set_error.set(Some(e));
                }
            }
        });
    };
    load_chapter(start_chapter, start_anchor.clone());

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
            if start_anchor.is_none()
                && saved_chapter.is_some()
                && saved_chapter == chapter_no.try_get_untracked()
            {
                set_pending_anchor.set(Some(cfi));
            }
        }
        set_position_known.set(true);
        set_book.set(detail);
        set_queued.set(api::pending_count().await);
    });

    // Column count follows the measured width; the class it sets changes
    // the page padding, so the width math runs a frame later.
    let measure = move |then: Box<dyn FnOnce()>| {
        let Some(el) = viewport_ref.get_untracked() else {
            return;
        };
        if el.client_width() <= 0 {
            return;
        }
        let columns = columns_for(window_width());
        set_spread.set(columns == 2);
        request_animation_frame(move || {
            let Some(el) = viewport_ref.try_get_untracked().flatten() else {
                return;
            };
            let width = f64::from(el.client_width());
            let padding = paddings(&el).left;
            let col = (width - 2.0 * padding * columns as f64) / columns as f64;
            set_column_width.set(col.max(120.0));
            then();
        });
    };

    let paginate = move || {
        let Some(el) = viewport_ref.try_get_untracked().flatten() else {
            return;
        };
        let width = el.client_width().max(1);
        let total = ((f64::from(el.scroll_width()) / f64::from(width)).ceil() as usize).max(1);
        set_pages.set(total);
        if let Some(cfi) = pending_anchor.try_get_untracked().flatten() {
            if cfi == ANCHOR_END {
                set_page.set(total - 1);
            } else if cfi == ANCHOR_SPOT {
                if let Some(p) = spot
                    .try_get_value()
                    .flatten()
                    .and_then(|s| page_of_spot(&el, s))
                {
                    set_page.set(p.min(total - 1));
                }
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
        cancel_flip();
        request_animation_frame(move || {
            measure(Box::new(move || request_animation_frame(paginate)));
        });
    });

    // The saved position may arrive after the chapter is laid out; apply it
    // then instead of waiting for the next chapter or preference change.
    Effect::new(move || {
        if pending_anchor.get().is_none() {
            return;
        }
        let shown = chapter.get_untracked().map(|c| c.chapter_number);
        if shown == Some(chapter_no.get_untracked()) && column_width.get_untracked() > 0.0 {
            request_animation_frame(paginate);
        }
    });

    let percent = Memo::new(move |_| {
        let (Some(b), Some(shown)) = (
            book.get(),
            chapter.with(|c| c.as_ref().map(|c| c.chapter_number)),
        ) else {
            return 0.0f32;
        };
        let total = b.chapters.len() as f32;
        if total == 0.0 {
            return 0.0;
        }
        let index = b
            .chapters
            .iter()
            .position(|c| c.chapter_number == shown)
            .unwrap_or(0) as f32;
        let fraction = (page.get() as f32 + 1.0) / pages.get().max(1) as f32;
        (((index + fraction) / total) * 100.0).clamp(0.0, 100.0)
    });

    // The position to save, or None while there is none yet: no chapter,
    // the stored one still unknown, or an anchor still waiting to apply.
    let position = move || -> Option<(String, ReadingProgressUpdateDto)> {
        if !position_known.try_get_untracked()? || pending_anchor.try_get_untracked()?.is_some() {
            return None;
        }
        let ch = chapter.try_get_untracked()??;
        Some((
            book_id.try_get_value()?,
            ReadingProgressUpdateDto {
                chapter_id: ch.id,
                last_anchor_cfi: anchor.try_get_untracked()?,
                completion_percentage: percent.try_get_untracked()?,
            },
        ))
    };

    let persist = move || {
        save_pending.try_set_value(false);
        let Some((id, update)) = position() else {
            return;
        };
        if !session.authed.get_untracked() {
            persist_guest(&id, &update);
            return;
        }
        spawn_local(async move {
            if api::save_progress(&id, &update).await.is_err() {
                let _ = api::queue_pending_progress(&id, &update).await;
            }
            set_queued.try_set(api::pending_count().await);
        });
    };

    let schedule_save = move || {
        save_pending.set_value(true);
        let generation = save_generation.get_untracked() + 1;
        set_save_generation.set(generation);
        set_timeout(
            move || {
                if save_generation.try_get_untracked() == Some(generation) {
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
        // Kept while it still lies on this page: rederived after a relayout,
        // it would walk back to whatever paragraph now opens the page.
        let kept = spot.get_value().and_then(|s| page_of_spot(&el, s)) == Some(current);
        if !kept {
            spot.set_value(spot_on(&el, current));
        }
        schedule_save();
    });

    let crossfade = move || {
        set_turning.set(true);
        set_timeout(move || set_turning.set(false), Duration::from_millis(140));
    };

    let geometry = move || {
        let el = viewport_ref.get_untracked()?;
        let mode = if spread.get_untracked() {
            Mode::Spread
        } else {
            Mode::Single
        };
        Some(Geometry::measure(&el, mode))
    };

    // Paper turn when motion is welcome and the page can be rebuilt; a plain
    // page change with a short crossfade otherwise.
    let turn_page = move |dir: Dir, from: usize, to: usize| {
        if !prefers_reduced_motion() {
            if let Some(geo) = geometry() {
                let plan = TurnPlan {
                    dir,
                    from_page: from,
                    to_page: Some(to),
                };
                let started =
                    flipper.with_value(|f| f.as_ref().is_some_and(|f| f.begin(plan, &geo, None)));
                if started {
                    return;
                }
            }
        }
        set_page.set(to);
        crossfade();
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
    let following_chapter = move || {
        let numbers = chapter_numbers();
        numbers
            .iter()
            .position(|n| *n == chapter_no.get_untracked())
            .and_then(|i| numbers.get(i + 1))
            .copied()
    };
    let preceding_chapter = move || {
        let numbers = chapter_numbers();
        numbers
            .iter()
            .position(|n| *n == chapter_no.get_untracked())
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| numbers.get(i))
            .copied()
    };

    let go_chapter = move |number: i32, anchor_hint: Option<String>| {
        set_menu_open.set(false);
        load_chapter(number, anchor_hint);
    };
    // Until a chapter is on screen there is nothing to finish or leave.
    let chapter_shown = move || chapter.with_untracked(Option::is_some);

    // Leaving a chapter forward shows its closing card the first time; the
    // next step forward, or the card's own button, moves on.
    let advance = move |number: i32| {
        if !chapter_shown() {
            return;
        }
        let current = chapter_no.get_untracked();
        let first_time = ends_shown.with_value(|shown| !shown.contains(&current));
        if !chapter_end.get_untracked() && first_time {
            ends_shown.update_value(|shown| {
                shown.insert(current);
            });
            cancel_flip();
            set_chapter_end.set(true);
            return;
        }
        go_chapter(number, None);
    };

    let next = move || {
        if flip_active() || finished.get_untracked() || !chapter_shown() {
            return;
        }
        let current = page.get_untracked();
        if current + 1 < pages.get_untracked() && !chapter_end.get_untracked() {
            turn_page(Dir::Forward, current, current + 1);
            return;
        }
        match following_chapter() {
            Some(n) => advance(n),
            // Without the book's chapter list the end of this chapter is
            // not known to be the end of the book.
            None if book.with_untracked(Option::is_some) => {
                set_finished.set(true);
                toasts.info(lang.get_untracked().text("book_end"));
            }
            None => {}
        }
    };

    let prev = move || {
        if flip_active() {
            return;
        }
        // Stepping back abandons a next chapter still on its way.
        if chapter_shown() {
            load_generation.update_value(|g| *g += 1);
        }
        if finished.get_untracked() {
            set_finished.set(false);
            return;
        }
        if chapter_end.get_untracked() {
            set_chapter_end.set(false);
            return;
        }
        let current = page.get_untracked();
        if current > 0 {
            turn_page(Dir::Back, current, current - 1);
            return;
        }
        if let Some(n) = preceding_chapter() {
            go_chapter(n, Some(ANCHOR_END.to_string()));
        }
    };

    let zone_at = move |x: f64| -> Option<(Zone, web_sys::DomRect)> {
        let frame = book_ref.get_untracked()?;
        let rect = frame.get_bounding_client_rect();
        let rel = (x - rect.left()) / rect.width().max(1.0);
        let zone = if rel < EDGE_ZONE {
            Zone::Left
        } else if rel > 1.0 - EDGE_ZONE {
            Zone::Right
        } else {
            Zone::Center
        };
        Some((zone, rect))
    };

    let on_pointer_down = move |ev: web_sys::PointerEvent| {
        if flip_active()
            || finished.get_untracked()
            || chapter_end.get_untracked()
            || menu_open.get_untracked()
        {
            return;
        }
        if starts_on_control(&ev) {
            return;
        }
        let x = f64::from(ev.client_x());
        let y = f64::from(ev.client_y());
        let Some((zone, rect)) = zone_at(x) else {
            return;
        };
        if y < rect.top() || y > rect.bottom() {
            return;
        }
        gesture.set_value(Some(Gesture {
            x0: x,
            y0: y,
            zone,
            touch: ev.pointer_type() == "touch",
            dragging: false,
            book_left: rect.left(),
            book_right: rect.right(),
            crossing: None,
        }));
    };

    // A finger may drag anywhere; a mouse only from the page edges, so
    // selecting text with the mouse keeps working in the middle.
    let on_pointer_move = move |ev: web_sys::PointerEvent| {
        let Some(mut g) = gesture.get_value() else {
            return;
        };
        let x = f64::from(ev.client_x());
        if g.dragging {
            flipper.with_value(|f| {
                if let Some(f) = f {
                    f.drag_to(x);
                }
            });
            return;
        }
        let dx = x - g.x0;
        let dy = f64::from(ev.client_y()) - g.y0;
        if dx.abs() < TAP_PX {
            return;
        }
        if dy.abs() > dx.abs() || (!g.touch && g.zone == Zone::Center) {
            gesture.set_value(None);
            return;
        }
        let dir = if dx < 0.0 { Dir::Forward } else { Dir::Back };
        let current = page.get_untracked();
        let total = pages.get_untracked();
        let to_page = match dir {
            Dir::Forward => (current + 1 < total).then_some(current + 1),
            Dir::Back => current.checked_sub(1),
        };
        if to_page.is_none() {
            g.crossing = match dir {
                Dir::Forward => following_chapter(),
                Dir::Back => preceding_chapter(),
            };
        }
        let started = match (geometry(), prefers_reduced_motion()) {
            (Some(geo), false) => flipper.with_value(|f| {
                f.as_ref().is_some_and(|f| {
                    f.begin(
                        TurnPlan {
                            dir,
                            from_page: current,
                            to_page,
                        },
                        &geo,
                        Some(DragStart {
                            x: g.x0,
                            book_left: g.book_left,
                            book_right: g.book_right,
                            touch: g.touch,
                        }),
                    )
                })
            }),
            _ => false,
        };
        g.dragging = true;
        gesture.set_value(Some(g));
        if started {
            if let Some(stage) = stage_ref.get_untracked() {
                let _ = stage.set_pointer_capture(ev.pointer_id());
            }
        }
    };

    let on_pointer_up = move |ev: web_sys::PointerEvent| {
        let Some(g) = gesture.get_value() else { return };
        gesture.set_value(None);
        let x = f64::from(ev.client_x());
        if g.dragging {
            let travel = (x - g.x0).abs();
            if let Some(number) = g.crossing.filter(|_| travel >= CHAPTER_SWIPE_PX) {
                cancel_flip();
                if number > chapter_no.get_untracked() {
                    advance(number);
                } else {
                    go_chapter(number, Some(ANCHOR_END.to_string()));
                }
                return;
            }
            if flip_active() {
                flipper.with_value(|f| {
                    if let Some(f) = f {
                        f.release();
                    }
                });
            } else if travel >= CHAPTER_SWIPE_PX {
                if x < g.x0 {
                    next();
                } else {
                    prev();
                }
            }
            return;
        }
        if !selection_is_collapsed() {
            return;
        }
        match g.zone {
            Zone::Left => prev(),
            Zone::Right => next(),
            Zone::Center => set_dim.update(|d| *d = !*d),
        }
    };

    let on_pointer_cancel = move |_: web_sys::PointerEvent| {
        if gesture.get_value().is_some_and(|g| g.dragging) {
            flipper.with_value(|f| {
                if let Some(f) = f {
                    f.release();
                }
            });
        }
        gesture.set_value(None);
    };

    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            set_menu_open.set(false);
            return;
        }
        let sheet_open = drawer_open.get_untracked()
            || search_open.get_untracked()
            || cards_open.get_untracked()
            || recap_open.get_untracked();
        if menu_open.get_untracked() || sheet_open || key_belongs_to_target(&ev) {
            return;
        }
        match ev.key().as_str() {
            "ArrowRight" | "PageDown" => {
                ev.prevent_default();
                next();
            }
            "ArrowLeft" | "PageUp" => {
                ev.prevent_default();
                prev();
            }
            " " => {
                ev.prevent_default();
                if ev.shift_key() {
                    prev();
                } else {
                    next();
                }
            }
            "Home" => {
                ev.prevent_default();
                cancel_flip();
                set_page.set(0);
            }
            "End" => {
                ev.prevent_default();
                cancel_flip();
                set_page.set(pages.get_untracked().saturating_sub(1));
            }
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());

    // Leaving inside the save debounce keeps the page just turned: an
    // in-app navigation saves as usual when the reader unmounts; closing the
    // tab uses a keepalive request the browser finishes after unload.
    on_cleanup(move || {
        if save_pending.try_get_value() == Some(true) {
            persist();
        }
    });
    let pagehide = window_event_listener(leptos::ev::pagehide, move |_| {
        if save_pending.get_value() {
            save_pending.set_value(false);
            if let Some((id, update)) = position() {
                if session.authed.get_untracked() {
                    api::save_progress_on_exit(&id, &update);
                } else {
                    persist_guest(&id, &update);
                }
            }
        }
    });
    on_cleanup(move || pagehide.remove());

    watch_selection(move || {
        set_selection.try_set(selected_chapter_text());
    });
    let save_selection = move |_| {
        let (Some(text), Some(ch), Ok(book_uuid)) = (
            selection.get_untracked(),
            chapter.get_untracked(),
            book_id.get_value().parse::<uuid::Uuid>(),
        ) else {
            return;
        };
        // The selectionchange that follows hides the chip after this click.
        clear_selection();
        spawn_local(async move {
            match keep_quote(book_uuid, ch.id, text).await {
                Ok(message) => toasts.info(lang.get_untracked().text(message)),
                Err(e) => toasts.error(crate::components::auth::friendly_error(
                    &e,
                    lang.get_untracked(),
                )),
            }
        });
    };

    let pick_chapter = Callback::new(move |number: i32| {
        close_after_event(drawer_open);
        if number != chapter_no.get_untracked() {
            go_chapter(number, None);
        }
    });
    let pick_hit = Callback::new(move |(number, paragraph): (i32, i32)| {
        close_after_event(search_open);
        let anchor = format!("p-{paragraph}");
        set_flash.set(Some((number, paragraph.max(1) as usize - 1)));
        if number == chapter_no.get_untracked() && chapter.get_untracked().is_some() {
            set_chapter_end.set(false);
            set_finished.set(false);
            set_pending_anchor.set(Some(anchor));
        } else {
            go_chapter(number, Some(anchor));
        }
    });
    // The paragraph a search hit opened glows briefly once its page is laid out.
    Effect::new(move || {
        let Some((number, index)) = flash.get() else {
            return;
        };
        let landed = chapter.get().is_some_and(|c| c.chapter_number == number)
            && pending_anchor.get().is_none()
            && pages.get() > 0;
        if !landed {
            return;
        }
        set_flash.set(None);
        request_animation_frame(move || {
            let Some(el) = viewport_ref.try_get_untracked().flatten() else {
                return;
            };
            if let Some(p) = paragraphs(&el).into_iter().nth(index) {
                let _ = p.class_list().add_1("search-flash");
                set_timeout(
                    move || {
                        let _ = p.class_list().remove_1("search-flash");
                    },
                    Duration::from_millis(2400),
                );
            }
        });
    });

    // The return target is set only once the new column width is in place;
    // set earlier, the anchor effect would paginate a half-updated layout.
    let relayout = move || {
        if !chapter_shown() {
            return;
        }
        cancel_flip();
        measure(Box::new(move || {
            if pending_anchor.try_get_untracked() == Some(None) {
                let target = if spot.try_get_value().flatten().is_some() {
                    ANCHOR_SPOT.to_string()
                } else {
                    anchor.get_untracked()
                };
                set_pending_anchor.set(Some(target));
            }
            request_animation_frame(paginate);
        }));
    };
    let resize = window_event_listener(leptos::ev::resize, move |_| relayout());
    on_cleanup(move || resize.remove());
    // The viewport's own size as well: a page scrollbar that comes and goes
    // during a rotation changes the page width with no window `resize`.
    Effect::new(move || {
        if let Some(el) = viewport_ref.get() {
            watch_size(&el, relayout);
        }
    });

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
    // The type panel is not modal, but a keyboard reader lands in it when it
    // opens and back on "Aa" when it closes and focus would fall to the page.
    let type_button = NodeRef::<leptos::html::Button>::new();
    Effect::new(move |was_open: Option<bool>| {
        let open = menu_open.get();
        if open {
            request_animation_frame(|| {
                // The chosen theme first; a selector list would match in
                // document order and always take the first swatch.
                let first = [".type-sheet .swatch.active", ".type-sheet button"]
                    .iter()
                    .find_map(|s| document().query_selector(s).ok().flatten())
                    .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
                if let Some(first) = first {
                    let _ = first.focus();
                }
            });
        } else if was_open == Some(true) {
            let lost = document()
                .active_element()
                .is_none_or(|el| el.tag_name().eq_ignore_ascii_case("body"));
            if lost {
                if let Some(button) = type_button.get_untracked() {
                    let _ = button.focus();
                }
            }
        }
        open
    });
    let chapter_end_title = move || {
        lang.get()
            .text_with("chapter_end_title", "n", &roman(chapter_no.get()))
    };
    let book_title = move || book.get().map(|b| b.title).unwrap_or_default();
    let chapter_head = move || {
        format!(
            "{} {} \u{00B7} {}",
            lang.get().text("chapter"),
            roman(chapter_no.get()),
            chapter.get().map(|c| c.title).unwrap_or_default()
        )
    };
    let folio_left = move || {
        if spread.get() {
            2 * page.get() + 1
        } else {
            page.get() + 1
        }
    };
    let folio_right = move || 2 * page.get() + 2;
    let page_status = move || {
        if spread.get() {
            lang.get()
                .text("page_range")
                .replace("{a}", &(2 * page.get() + 1).to_string())
                .replace("{b}", &(2 * page.get() + 2).to_string())
                .replace("{n}", &(2 * pages.get()).to_string())
        } else {
            lang.get()
                .text("page_of")
                .replace("{a}", &(page.get() + 1).to_string())
                .replace("{b}", &pages.get().to_string())
        }
    };
    let restart = move |_| {
        set_finished.set(false);
        if let Some(first) = chapter_numbers().first().copied() {
            go_chapter(first, None);
        }
    };

    view! {
        <div class="reader" data-reading=move || prefs.get().theme.attr() class:spread=move || spread.get() class:dim=move || dim.get()>
            <header class="reader-bar">
                <a href=format!("/book/{}", book_id.get_value()) class="btn-icon" aria-label=move || lang.get().text("reader_back")>{icons::back()}</a>
                // The name starts with what the button shows, so a voice
                // command naming the visible chapter reaches it.
                <button class="reader-title" aria-label=move || {
                    let shown = format!("{} {} {}", lang.get().text("chapter"), roman(chapter_no.get()), chapter.get().map(|c| c.title).unwrap_or_default());
                    format!("{}, {}", shown.trim_end(), lang.get().text("open_chapters"))
                } aria-expanded=move || drawer_open.get() on:click=move |_| drawer_open.set(true)>
                    <span class="kicker">{move || format!("{} {}", lang.get().text("chapter"), roman(chapter_no.get()))}</span>
                    <span class="label">{move || chapter.get().map(|c| c.title).unwrap_or_default()}{icons::arrow_down()}</span>
                </button>
                <div class="reader-tools">
                    {move || (chapter_no.get() > 1).then(|| {
                        let last = chapter_no.get() - 1;
                        let range = if last == 1 { roman(1) } else { format!("I\u{2013}{}", roman(last)) };
                        let label = Signal::derive(move || lang.get().text_with("recap_range", "range", &range));
                        view! {
                            <button class="recap-chip" aria-label=label on:click=move |_| recap_open.set(true)>
                                {icons::recap()}
                                <span>{label}</span>
                            </button>
                        }
                    })}
                    <button class="btn-icon btn-search" aria-label=move || lang.get().text("book_search") on:click=move |_| search_open.set(true)>{icons::search()}</button>
                    <button class="btn-icon btn-type" node_ref=type_button aria-label=move || lang.get().text("reader_settings") aria-expanded=move || menu_open.get() on:click=toggle_menu>"Aa"</button>
                </div>
            </header>

            <div
                node_ref=stage_ref
                class="reader-stage"
                on:pointerdown=on_pointer_down
                on:pointermove=on_pointer_move
                on:pointerup=on_pointer_up
                on:pointercancel=on_pointer_cancel
            >
                <button class="reader-zone reader-zone-prev" aria-label=move || lang.get().text("prev_page") on:click=move |_| prev()>{icons::back()}</button>
                <div node_ref=book_ref class="book">
                    <span class="page-head page-head-left" aria-hidden="true">{book_title}</span>
                    <span class="page-head page-head-right" aria-hidden="true">{chapter_head}</span>
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
                                Some(err) => {
                                    let not_found = api::is_not_found(&err);
                                    let message = load_failure(&err);
                                    view! {
                                        <div class="reader-end">
                                            <p>{move || lang.get().text(message)}</p>
                                            {(!not_found).then(|| view! {
                                                <button class="btn btn-primary" on:click=move |_| load_chapter(chapter_no.get_untracked(), None)>
                                                    {move || lang.get().text("retry")}
                                                </button>
                                            })}
                                        </div>
                                    }.into_any()
                                }
                            },
                            Some(c) => view! { <div class="chapter-body" lang=move || book.get().and_then(|b| crate::i18n::text_lang(&b.language)) inner_html=c.html_content></div> }.into_any(),
                        }}
                    </div>
                    <span class="spine" aria-hidden="true"></span>
                    <span class="folio folio-left" aria-hidden="true">{folio_left}</span>
                    <span class="folio folio-right" aria-hidden="true">{folio_right}</span>
                    <span class="corner" aria-hidden="true"></span>
                    <div node_ref=flip_ref class="flip-layer" aria-hidden="true"></div>
                    {move || chapter_end.get().then(|| view! {
                        <section class="finale chapter-end" aria-label=chapter_end_title>
                            <h2 class="display">{chapter_end_title}</h2>
                            <p>{move || lang.get().text("chapter_end_body")}</p>
                            <div class="sheet-actions centered">
                                <button class="btn btn-primary" on:click=move |_| {
                                    if let Some(n) = following_chapter() {
                                        advance(n);
                                    }
                                }>{move || lang.get().text("next_chapter")}</button>
                                <button class="btn btn-ghost" on:click=move |_| cards_open.set(true)>{icons::cards()}{move || lang.get().text("atomic_cards")}</button>
                            </div>
                        </section>
                    })}
                    {move || finished.get().then(|| view! {
                        <section class="finale" aria-label=move || lang.get().text("book_finished_title")>
                            <img src="/assets/cozy-reader-armchair-owl.webp" alt="" width="640" height="640"/>
                            <h2 class="display">{move || lang.get().text("book_finished_title")}</h2>
                            <p>{move || lang.get().text_with("book_finished_body", "title", &book_title())}</p>
                            <div class="sheet-actions centered">
                                <a href="/" class="btn btn-primary">{move || lang.get().text("next_book")}</a>
                                <a href="/me" class="btn btn-ghost">{move || lang.get().text("saved_quotes")}</a>
                                <button class="btn btn-ghost" on:click=restart>{move || lang.get().text("read_again")}</button>
                            </div>
                        </section>
                    })}
                </div>
                <button class="reader-zone reader-zone-next" aria-label=move || lang.get().text("next_page") on:click=move |_| next()>{icons::arrow_right()}</button>
                {move || menu_open.get().then(|| view! { <TypeSheet prefs=prefs/> })}
            </div>

            <footer class="reader-footer">
                <ProgressBar percent=percent label=Signal::derive(move || lang.get().text("reading_progress")) thin=true/>
                <div class="reader-status">
                    <span>{page_status}</span>
                    <span>
                        {move || if page.get() + 1 >= pages.get() {
                            lang.get().text("chapter_last_page")
                        } else {
                            lang.get().text_n("minutes_left", i64::from(minutes_left().max(1)))
                        }}
                        {move || offline.get().then(|| view! { <span class="badge">{format!(" \u{00B7} {}", lang.get().text("offline_copy"))}</span> })}
                        {move || (queued.get() > 0).then(|| view! { <span class="badge">{format!(" \u{00B7} {}", lang.get().text_with("queued", "n", &queued.get().to_string()))}</span> })}
                    </span>
                </div>
            </footer>

            {move || view! { <CatchupRecap book_id=book_id.get_value() chapter=chapter_no.get() show=recap_open/> }}
            {move || view! { <AtomicCards book_id=book_id.get_value() chapter=chapter_no.get() show=cards_open/> }}
            <ChapterDrawer book=book current=chapter_no show=drawer_open on_pick=pick_chapter/>
            <BookSearch book_id=book_id.get_value() show=search_open on_pick=pick_hit/>
            {move || selection.get().is_some().then(|| view! {
                <button class="btn btn-primary quote-chip" on:mousedown=|ev| ev.prevent_default() on:click=save_selection>
                    {move || lang.get().text("save_selection")}
                </button>
            })}
            {move || streak.get().map(|days| view! {
                <div class="streak-toast" role="status">{format!("\u{2756} {}", lang.get().text_n("streak_days", i64::from(days)))}</div>
            })}
            <ToastStack toasts=toasts/>
        </div>
    }
}
