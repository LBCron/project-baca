//! Home page: live catalog grid, debounced search, cursor infinite scroll,
//! continue-reading card (Sub-Task 6.3).

use crate::api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{ActiveProgressDto, BookCatalogQuery, BookSearchQuery, BookSummaryDto};
use std::time::Duration;

async fn fetch_catalog(cursor: Option<String>) -> Result<Vec<BookSummaryDto>, String> {
    api::catalog(&BookCatalogQuery {
        cursor: cursor.as_deref().and_then(|c| c.parse::<uuid::Uuid>().ok()),
        limit: Some(20),
        ..Default::default()
    })
    .await
}

#[component]
pub fn HomePage(lang: ReadSignal<String>, set_lang: WriteSignal<String>) -> impl IntoView {
    let (books, set_books) = signal(Vec::<BookSummaryDto>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (query, set_query) = signal(String::new());
    let (progress, set_progress) = signal(Vec::<ActiveProgressDto>::new());

    let load_more = move || {
        if loading.get_untracked() {
            return;
        }
        set_loading.set(true);
        let cur = cursor.get_untracked();
        spawn_local(async move {
            if let Ok(mut page) = fetch_catalog(cur).await {
                if let Some(last) = page.last() {
                    set_cursor.set(Some(last.id.to_string()));
                }
                set_books.update(|all| all.append(&mut page));
            }
            set_loading.set(false);
        });
    };

    spawn_local(async move {
        if let Ok(items) = api::active_progress().await {
            set_progress.set(items);
        }
    });
    load_more();

    let on_search = move |ev| {
        let value = event_target_value(&ev);
        set_query.set(value.clone());
        set_timeout(
            move || {
                if value.trim().is_empty() {
                    set_books.set(Vec::new());
                    set_cursor.set(None);
                    load_more();
                    return;
                }
                set_loading.set(true);
                spawn_local(async move {
                    let req = BookSearchQuery {
                        q: value,
                        limit: Some(10),
                        ..Default::default()
                    };
                    if let Ok(results) = api::search(&req).await {
                        set_books.set(
                            results
                                .into_iter()
                                .map(|r| BookSummaryDto {
                                    id: r.id,
                                    title: r.title,
                                    author: r.author,
                                    language: r.language,
                                    primary_theme: r.primary_theme,
                                    sub_theme: None,
                                    description: String::new(),
                                    cover_url: r.cover_url,
                                    total_words: 0,
                                    estimated_reading_minutes: r.estimated_reading_minutes,
                                    publication_year: None,
                                    license: String::new(),
                                    tags: Vec::new(),
                                })
                                .collect(),
                        );
                    }
                    set_loading.set(false);
                });
            },
            Duration::from_millis(250),
        );
    };

    let toggle_lang = move |_| {
        set_lang.update(|l| {
            if l == "EN" {
                *l = "ID".to_string();
            } else {
                *l = "EN".to_string();
            }
        });
    };

    view! {
        <div class="app-container">
            <header class="header-vintage">
                <a href="/" class="brand-title">
                    <span class="brand-ornament">"❖"</span>
                    <span>"Project Baca"</span>
                </a>
                <nav class="nav-links">
                    <a href="/" class="nav-link">"Catalog"</a>
                    <button class="lang-switch" on:click=toggle_lang>
                        {move || format!("[ {} ]", lang.get())}
                    </button>
                </nav>
            </header>

            <section class="hero-vintage">
                <div class="hero-subtitle">"Public Domain Classical Literature"</div>
                <h1 class="hero-heading">"The Timelessness of Words in Classic Print"</h1>
                <p class="hero-desc">
                    <input
                        type="search"
                        class="search-bar"
                        placeholder="Search title or author…"
                        prop:value=move || query.get()
                        on:input=on_search
                    />
                </p>
            </section>

            {move || {
                let items = progress.get();
                (!items.is_empty()).then(|| view! {
                    <section class="continue-reading">
                        <div class="catalog-section-title"><span>"Continue Reading"</span></div>
                        <div class="book-grid">
                            {items.into_iter().map(|p| view! {
                                <div class="book-card">
                                    <div class="book-tag">
                                        {format!("Ch. {} — {:.1}%", p.chapter_number, p.completion_percentage)}
                                    </div>
                                    <h2 class="book-title">{p.book_title.clone()}</h2>
                                    <div class="book-author">{p.book_author.clone()}</div>
                                    <div class="book-card-footer">
                                        <span>{p.chapter_title.clone()}</span>
                                        <a href=format!("/book/{}", p.book_id) class="btn-read">
                                            <span>"Resume"</span>
                                            <span>"→"</span>
                                        </a>
                                    </div>
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                    </section>
                })
            }}

            <main>
                <div class="catalog-section-title">
                    <span>"Catalog"</span>
                </div>
                <div class="book-grid">
                    {move || books.get().into_iter().map(|b| view! {
                        <div class="book-card">
                            <div>
                                <div class="book-tag">{b.primary_theme.clone()}</div>
                                <h2 class="book-title">{b.title.clone()}</h2>
                                <div class="book-author">{b.author.clone()}</div>
                                <p class="book-synopsis">{b.description.clone()}</p>
                            </div>
                            <div class="book-card-footer">
                                <span>{format!("~{} min", b.estimated_reading_minutes)}</span>
                                <a href=format!("/book/{}", b.id) class="btn-read">
                                    <span>"Read"</span>
                                    <span>"→"</span>
                                </a>
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
                <div class="load-more">
                    <button class="btn-read" on:click=move |_| load_more() disabled=move || loading.get()>
                        {move || if loading.get() { "Loading…" } else { "Load More" }}
                    </button>
                </div>
            </main>

            <footer class="footer-vintage">
                <div class="footer-fleuron">"❖"</div>
                <p>"Project Baca — Literary manuscripts are in the public domain. Source code licensed under MIT / Apache-2.0."</p>
            </footer>
        </div>
    }
}
