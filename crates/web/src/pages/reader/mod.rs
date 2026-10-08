//! Paginated reader laid out as an open book: chapter HTML flows into CSS
//! columns one viewport wide (two columns on wide screens), pages turn like
//! paper, the position is anchored to the first visible paragraph and saved
//! after every turn, and heartbeats report real elapsed time.

mod flip;
mod layout;

use crate::api;
use crate::components::anchor::prefers_reduced_motion;
use crate::components::icons;
use crate::components::insights::CatchupRecap;
use crate::components::progress::ProgressBar;
use crate::components::session::use_session;
use crate::components::toast::{use_toasts, ToastStack};
use crate::format::{reading_minutes, roman};
use crate::i18n::use_lang;
use crate::storage::{self, OfflineChapterRecord};
use crate::theme::{ReaderPrefs, ReadingTheme, FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS};
use flip::{Dir, DragStart, Flipper, Geometry, Mode, TurnPlan};
use layout::{anchor_index, columns_for, paddings, page_of, paragraphs};
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
/// Pointer travel below this is a tap; beyond it, a drag.
const TAP_PX: f64 = 6.0;
/// Travel that turns a chapter boundary on release.
const CHAPTER_SWIPE_PX: f64 = 48.0;
/// Outer share of the page that acts as a tap zone on either side.
const EDGE_ZONE: f64 = 0.22;
/// Anchor hint meaning "open on the last page" (stepping back a chapter).
const ANCHOR_END: &str = "end";

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

fn selection_is_collapsed() -> bool {
    web_sys::window()
        .and_then(|w| w.get_selection().ok().flatten())
        .map(|s| s.is_collapsed())
        .unwrap_or(true)
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
    let (turning, set_turning) = signal(false);
    let (streak, set_streak) = signal(None::<i32>);
    let (queued, set_queued) = signal(0usize);
    let (save_generation, set_save_generation) = signal(0u32);
    let wants_recap = use_query_map().get_untracked().get("recap").is_some();
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
        set_pending_anchor.set(anchor_hint);
        set_page.set(0);
        set_finished.set(false);
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
            if saved_chapter.is_some() && saved_chapter == chapter_no.try_get_untracked() {
                set_pending_anchor.set(Some(cfi));
            }
        }
        set_book.set(detail);
        set_queued.set(api::pending_count().await);
    });

    // Column count follows the measured width; the class it sets changes
    // the page padding, so the width math runs a frame later.
    let measure = move |then: Box<dyn FnOnce()>| {
        let Some(el) = viewport_ref.get_untracked() else {
            return;
        };
        let width = el.client_width();
        if width <= 0 {
            return;
        }
        let columns = columns_for(width);
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
        set_chapter_no.set(number);
        set_menu_open.set(false);
        load_chapter(number, anchor_hint);
        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
            let url = format!("/read/{}?chapter={number}", book_id.get_value());
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    };

    let next = move || {
        if flip_active() || finished.get_untracked() {
            return;
        }
        let current = page.get_untracked();
        if current + 1 < pages.get_untracked() {
            turn_page(Dir::Forward, current, current + 1);
            return;
        }
        match following_chapter() {
            Some(n) => go_chapter(n, None),
            None => {
                set_finished.set(true);
                toasts.info(lang.get_untracked().text("book_end"));
            }
        }
    };

    let prev = move || {
        if flip_active() {
            return;
        }
        if finished.get_untracked() {
            set_finished.set(false);
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
        if flip_active() || finished.get_untracked() || menu_open.get_untracked() {
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
                let hint = (number < chapter_no.get_untracked()).then(|| ANCHOR_END.to_string());
                go_chapter(number, hint);
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
        if menu_open.get_untracked() || recap_open.get_untracked() {
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

    let resize = window_event_listener(leptos::ev::resize, move |_| {
        cancel_flip();
        set_pending_anchor.set(Some(anchor.get_untracked()));
        measure(Box::new(move || request_animation_frame(paginate)));
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
    let chapters_href = move || format!("/book/{}#daftar-bab", book_id.get_value());
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
                <a href=chapters_href class="reader-title" title=move || lang.get().text("chapters")>
                    <span class="kicker">{move || format!("{} {}", lang.get().text("chapter"), roman(chapter_no.get()))}</span>
                    <span class="label">{move || chapter.get().map(|c| c.title).unwrap_or_default()}{icons::arrow_down()}</span>
                </a>
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
                    <button class="btn-icon btn-type" aria-label=move || lang.get().text("reader_settings") aria-expanded=move || menu_open.get() on:click=toggle_menu>"Aa"</button>
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
                                    let message = if not_found {
                                        "not_found"
                                    } else if api::is_online() {
                                        "chapter_load_failed"
                                    } else {
                                        "offline_notice"
                                    };
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
                            Some(c) => view! { <div class="chapter-body" inner_html=c.html_content></div> }.into_any(),
                        }}
                    </div>
                    <span class="spine" aria-hidden="true"></span>
                    <span class="folio folio-left" aria-hidden="true">{folio_left}</span>
                    <span class="folio folio-right" aria-hidden="true">{folio_right}</span>
                    <span class="corner" aria-hidden="true"></span>
                    <div node_ref=flip_ref class="flip-layer" aria-hidden="true"></div>
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
            <ToastStack toasts=toasts/>
        </div>
    }
}
