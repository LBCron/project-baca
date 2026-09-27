//! Book overview, chapter list, offline save.

use crate::components::insights::{AtomicCards, QuoteFinder};
use crate::{api, storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared::BookDetailDto;

#[component]
pub fn BookPage() -> impl IntoView {
    let params = use_params_map();
    let book_id = move || params.get().get("id").unwrap_or_default();
    let (book, set_book) = signal(None::<BookDetailDto>);
    let (error, set_error) = signal(None::<String>);
    let (saving, set_saving) = signal(false);
    let (saved, set_saved) = signal(false);
    let quotes_open = RwSignal::new(false);
    let cards_open = RwSignal::new(false);

    {
        let id = book_id();
        spawn_local(async move {
            match api::book_detail(&id).await {
                Ok(detail) => set_book.set(Some(detail)),
                Err(e) => set_error.set(Some(e)),
            }
        });
    }

    let save_offline = move |_| {
        let id = book_id();
        set_saving.set(true);
        spawn_local(async move {
            match api::offline_bundle(&id).await {
                Ok(bundle) => {
                    let book_ok = storage::put(storage::STORE_OFFLINE_BOOKS, &bundle.book)
                        .await
                        .is_ok();
                    let mut chapters_ok = true;
                    for chapter in &bundle.chapters {
                        let record = serde_json::json!({
                            "chapter_id": chapter.id,
                            "book_id": chapter.book_id,
                            "chapter": chapter,
                        });
                        if storage::put(storage::STORE_OFFLINE_CHAPTERS, &record)
                            .await
                            .is_err()
                        {
                            chapters_ok = false;
                            break;
                        }
                    }
                    set_saved.set(book_ok && chapters_ok);
                    if !(book_ok && chapters_ok) {
                        set_error.set(Some("Failed to cache bundle offline.".to_string()));
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="app-container">
            <header class="header-vintage">
                <a href="/" class="brand-title">
                    <span class="brand-ornament">"❖"</span>
                    <span>"Project Baca"</span>
                </a>
            </header>
            {move || match book.get() {
                None => match error.get() {
                    None => view! { <p class="hero-desc">"Loading manuscript…"</p> }.into_any(),
                    Some(e) => view! { <p class="hero-desc">{format!("Failed to load: {e}")}</p> }.into_any(),
                },
                Some(b) => view! {
                    <section class="book-overview">
                        <div class="book-tag">{b.primary_theme.clone()}</div>
                        <h1 class="hero-heading">{b.title.clone()}</h1>
                        <div class="book-author">{format!("{} ({})", b.author.clone(), b.publication_year.unwrap_or(0))}</div>
                        <p class="hero-desc">{b.description.clone()}</p>
                        <div class="book-meta">
                            <span>{format!("~{} min", b.estimated_reading_minutes)}</span>
                            <span>{format!("{} words", b.total_words)}</span>
                            {b.tags.into_iter().map(|t| view! { <span class="book-tag">{t}</span> }).collect::<Vec<_>>()}
                        </div>
                        <div class="book-actions">
                            {b.chapters.first().map(|first| {
                                let href = format!("/read/{}?chapter={}", b.id, first.chapter_number);
                                view! { <a href=href class="btn-read"><span>"Start Reading"</span><span>"→"</span></a> }
                            })}
                            <button class="btn-read" on:click=save_offline disabled=move || saving.get() || saved.get()>
                                {move || if saved.get() { "Saved Offline" } else if saving.get() { "Saving…" } else { "Save Offline" }}
                            </button>
                            <button class="lang-switch" on:click=move |_| quotes_open.set(true)>"Quote Finder"</button>
                            <button class="lang-switch" on:click=move |_| cards_open.set(true)>"Atomic Cards"</button>
                        </div>
                        <QuoteFinder book_id=b.id.to_string() show=quotes_open/>
                        {b.chapters.first().map(|first| {
                            view! { <AtomicCards book_id=b.id.to_string() chapter=first.chapter_number show=cards_open/> }
                        })}
                        <div class="catalog-section-title"><span>"Chapters"</span></div>
                        <ol class="chapter-list">
                            {b.chapters.into_iter().map(|c| {
                                let href = format!("/read/{}?chapter={}", b.id, c.chapter_number);
                                view! {
                                    <li class="chapter-row">
                                        <a href=href>{format!("{}. {}", c.chapter_number, c.title)}</a>
                                        <span>{format!("{} words", c.word_count)}</span>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ol>
                    </section>
                }.into_any(),
            }}
        </div>
    }
}
