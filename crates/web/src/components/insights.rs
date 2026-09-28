//! Quote finder, atomic cards, recap, PNG export.

use crate::api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{QuoteSearchRequest, QuoteSearchResultDto};
use wasm_bindgen::JsCast;

fn export_quote_png(quote: &str, source: &str) {
    let document = match api::window_document() {
        Some(d) => d,
        None => return,
    };
    let canvas = match document.create_element("canvas") {
        Ok(el) => el.dyn_into::<web_sys::HtmlCanvasElement>().ok(),
        Err(_) => None,
    };
    let canvas = match canvas {
        Some(c) => c,
        None => return,
    };
    canvas.set_width(800);
    canvas.set_height(500);
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok());
    let ctx = match ctx {
        Some(c) => c,
        None => return,
    };
    ctx.set_fill_style_str("#F7F4EE");
    ctx.fill_rect(0.0, 0.0, 800.0, 500.0);
    ctx.set_fill_style_str("#9D5A3C");
    ctx.set_font("48px serif");
    ctx.fill_text("❖", 376.0, 80.0).ok();
    ctx.set_fill_style_str("#231D19");
    ctx.set_font("italic 28px Georgia, serif");
    let words: Vec<&str> = quote.split_whitespace().collect();
    let mut line = String::new();
    let mut y = 160.0;
    for word in words {
        let trial = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        let width = ctx.measure_text(&trial).map(|m| m.width()).unwrap_or(0.0);
        if width > 680.0 {
            ctx.fill_text(&line, 60.0, y).ok();
            y += 42.0;
            line = word.to_string();
        } else {
            line = trial;
        }
        if y > 380.0 {
            break;
        }
    }
    if !line.is_empty() && y <= 400.0 {
        ctx.fill_text(&line, 60.0, y).ok();
    }
    ctx.set_fill_style_str("#645A56");
    ctx.set_font("20px Georgia, serif");
    ctx.fill_text(source, 60.0, 450.0).ok();
    let url = canvas.to_data_url().unwrap_or_default();
    if url.is_empty() {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(doc) = window.document() else {
        return;
    };
    if let Ok(a) = doc.create_element("a") {
        let _ = a.set_attribute("href", &url);
        let _ = a.set_attribute("download", "quote-card.png");
        if let Some(anchor) = a.dyn_ref::<web_sys::HtmlAnchorElement>() {
            anchor.click();
        }
    }
}

#[component]
pub fn QuoteFinder(book_id: String, show: RwSignal<bool>) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (results, set_results) = signal(Vec::<QuoteSearchResultDto>::new());
    let (busy, set_busy) = signal(false);

    let book_for_search = StoredValue::new(book_id.clone());
    let book_for_rows = StoredValue::new(book_id);

    let run = move |_| {
        let q = query.get_untracked();
        if q.trim().len() < 3 {
            return;
        }
        set_busy.set(true);
        let book = book_for_search.get_value();
        spawn_local(async move {
            let req = QuoteSearchRequest {
                query: q,
                limit: Some(5),
            };
            if let Ok(items) = api::quote_search(&book, &req).await {
                set_results.set(items);
            }
            set_busy.set(false);
        });
    };
    view! {
        {move || show.get().then(|| view! {
            <div class="modal-backdrop">
                <div class="modal-vintage modal-wide">
                    <div class="catalog-section-title"><span>"Quote Finder"</span></div>
                    <input type="search" class="search-bar" placeholder="Concept or emotion…"
                        prop:value=move || query.get()
                        on:input=move |ev| set_query.set(event_target_value(&ev))/>
                    <div class="book-actions">
                        <button class="btn-read" on:click=run disabled=move || busy.get()>
                            {move || if busy.get() { "Searching…" } else { "Search" }}
                        </button>
                        <button class="btn-ghost" on:click=move |_| show.set(false)>"Close"</button>
                    </div>
                    <ol class="quote-results">
                        {move || {
                            let items = results.get();
                            (items.is_empty() && !busy.get()).then(|| view! {
                                <li class="quote-row"><p class="book-synopsis">"No quotes yet — this copy has no indexed passages."</p></li>
                            })
                        }}
                        {move || (!crate::api::is_authed() && !results.get().is_empty()).then(|| view! {
                            <li class="quote-row"><p class="book-synopsis">"Sign in to save quotes to your shelf."</p></li>
                        })}
                        {move || results.get().into_iter().map(|r| {
                            let book = book_for_rows.get_value();
                            let jump = format!("/read/{book}?chapter={}", r.chapter_number);
                            let text = r.content.clone();
                            let label = format!("Ch. {} — {:.0}%", r.chapter_number, r.similarity_score * 100.0);
                            let save_text = r.content.clone();
                            let save_book = book.clone();
                            let save_chapter = r.chapter_number;
                            view! {
                                <li class="quote-row">
                                    <div class="book-tag">{label.clone()}</div>
                                    <p>{r.content.clone()}</p>
                                    <div class="book-actions">
                                        <a href=jump class="btn-read"><span>"Jump to Reader"</span></a>
                                        <button class="btn-ghost" on:click=move |_| {
                                            let text = save_text.clone();
                                            let book = save_book.clone();
                                            spawn_local(async move {
                                                let chapter_id = api::chapter(&book, save_chapter).await.map(|c| c.id);
                                                if let (Ok(book_id), Ok(chapter_id)) = (book.parse::<uuid::Uuid>(), chapter_id) {
                                                    let _ = api::save_quote(&shared::SaveQuoteRequest {
                                                        book_id,
                                                        chapter_id,
                                                        quote_text: text,
                                                    }).await;
                                                }
                                            });
                                        }>"Save"</button>
                                        <button class="btn-ghost" on:click=move |_| export_quote_png(&text, &label)>"Export PNG"</button>
                                    </div>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ol>
                </div>
            </div>
        })}
    }
}

#[component]
pub fn AtomicCards(book_id: String, chapter: i32, show: RwSignal<bool>) -> impl IntoView {
    let (body, set_body) = signal(None::<String>);
    spawn_local(async move {
        match api::atomic_cards(&book_id, &chapter.to_string()).await {
            Ok(dto) => set_body.set(Some(dto.cards.to_string())),
            Err(e) => set_body.set(Some(format!("Unavailable: {e}"))),
        }
    });
    view! {
        {move || show.get().then(|| view! {
            <div class="modal-backdrop">
                <div class="modal-vintage">
                    <div class="catalog-section-title"><span>{format!("Atomic Insights — Ch. {chapter}")}</span></div>
                    <p class="hero-desc">{move || body.get().unwrap_or_else(|| "Loading…".to_string())}</p>
                    <div class="book-actions">
                        <button class="btn-ghost" on:click=move |_| show.set(false)>"Close"</button>
                    </div>
                </div>
            </div>
        })}
    }
}

#[component]
pub fn CatchupRecap(book_id: String, chapter: i32, show: RwSignal<bool>) -> impl IntoView {
    let (body, set_body) = signal(None::<String>);
    spawn_local(async move {
        match api::chapter_recap(&book_id, &chapter.to_string()).await {
            Ok(dto) => set_body.set(Some(dto.recap.to_string())),
            Err(e) => set_body.set(Some(format!("Unavailable: {e}"))),
        }
    });
    view! {
        {move || show.get().then(|| view! {
            <div class="modal-backdrop">
                <div class="modal-vintage">
                    <div class="catalog-section-title"><span>"Catch-up Recap"</span></div>
                    <p class="hero-desc">{move || body.get().unwrap_or_else(|| "Loading…".to_string())}</p>
                    <div class="book-actions">
                        <button class="btn-ghost" on:click=move |_| show.set(false)>"Close"</button>
                    </div>
                </div>
            </div>
        })}
    }
}
