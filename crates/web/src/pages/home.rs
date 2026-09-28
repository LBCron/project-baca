//! Rotaria home: brand hero with art carousel, live catalog,
//! continue reading. All strings via i18n dictionary.

use crate::api;
use crate::components::auth::AuthModal;
use crate::i18n::{apply_lang, lang_signal};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{ActiveProgressDto, BookCatalogQuery, BookSearchQuery, BookSummaryDto};
use std::time::Duration;

const HERO_ART: [(&str, &str); 5] = [
    (
        "/assets/library-bookshelf-ladder.png",
        "Reader climbing a library ladder",
    ),
    (
        "/assets/cozy-reader-armchair-owl.png",
        "Reader in an armchair with tea and an owl",
    ),
    (
        "/assets/manuscript-inspection-clothesline.png",
        "Writer drying manuscript pages on a line",
    ),
    (
        "/assets/admin-sorting-pigeonholes.png",
        "Archivist sorting mail into pigeonholes",
    ),
    (
        "/assets/retro-rocket-discovery.png",
        "Victorian explorers in a retro rocket",
    ),
];

async fn fetch_catalog(cursor: Option<String>) -> Result<Vec<BookSummaryDto>, String> {
    api::catalog(&BookCatalogQuery {
        cursor: cursor.as_deref().and_then(|c| c.parse::<uuid::Uuid>().ok()),
        limit: Some(20),
        ..Default::default()
    })
    .await
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (lang, set_lang) = lang_signal();
    let (books, set_books) = signal(Vec::<BookSummaryDto>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (query, set_query) = signal(String::new());
    let (progress, set_progress) = signal(None::<ActiveProgressDto>);
    let (slide, set_slide) = signal(0usize);
    let (paused, set_paused) = signal(false);

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
        if let Ok(item) = api::active_progress().await {
            set_progress.set(item);
        }
    });
    load_more();

    Effect::new(move || {
        let reduced = web_sys::window()
            .and_then(|w| {
                w.match_media("(prefers-reduced-motion: reduce)")
                    .ok()
                    .flatten()
            })
            .map(|m| m.matches())
            .unwrap_or(false);
        if reduced {
            set_paused.set(true);
            return;
        }
        set_interval(
            move || {
                if !paused.get_untracked() {
                    set_slide.update(|s| *s = (*s + 1) % HERO_ART.len());
                }
            },
            Duration::from_millis(7000),
        );
    });

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

    let auth_open = RwSignal::new(false);
    let authed = Callback::new(move |_| {});

    view! {
        <div class="app-container">
            <header class="header-vintage">
                <a href="/" class="brand-title">
                    <img src="/assets/rotaria-windmill.svg" alt="Rotaria" class="brand-mark"/>
                    <span>"Rotaria"</span>
                </a>
                <nav class="nav-links">
                    <a href="/" class="nav-link">{move || lang.get().text("catalog")}</a>
                    <button class="nav-link" on:click=move |_| auth_open.set(true)>
                        {move || lang.get().text("sign_in")}
                    </button>
                    <div class="lang-toggle" role="group" aria-label="Language">
                        <button
                            class="lang-opt"
                            class:active=move || lang.get() == crate::i18n::Lang::Id
                            on:click=move |_| apply_lang(set_lang, crate::i18n::Lang::Id)
                        >"ID"</button>
                        <button
                            class="lang-opt"
                            class:active=move || lang.get() == crate::i18n::Lang::En
                            on:click=move |_| apply_lang(set_lang, crate::i18n::Lang::En)
                        >"EN"</button>
                    </div>
                </nav>
            </header>
            <AuthModal show=auth_open on_authed=authed/>

            <section class="hero-vintage">
                <div class="hero-copy">
                    <div class="hero-subtitle">{move || lang.get().text("hero_sub")}</div>
                    <h1 class="hero-heading">
                        {move || lang.get().text("hero_head_a")} <em>{move || lang.get().text("hero_head_b")}</em>
                    </h1>
                    <p class="hero-desc">{move || lang.get().text("hero_desc")}</p>
                    <p class="hero-tagline">{move || lang.get().text("tagline")}</p>
                    <input
                        type="search"
                        class="search-bar"
                        placeholder=move || lang.get().text("search_ph")
                        prop:value=move || query.get()
                        on:input=on_search
                    />
                </div>
                <div
                    class="hero-art"
                    on:mouseenter=move |_| set_paused.set(true)
                    on:mouseleave=move |_| set_paused.set(false)
                    on:focusin=move |_| set_paused.set(true)
                    on:focusout=move |_| set_paused.set(false)
                >
                    {move || {
                        let (src, alt) = HERO_ART[slide.get()];
                        view! {
                            <img src=src alt=alt class="hero-slide"/>
                        }
                    }}
                    <div class="hero-dots" role="tablist" aria-label="Hero art">
                        {HERO_ART.iter().enumerate().map(|(i, _)| {
                            let active = move || slide.get() == i;
                            view! {
                                <button
                                    role="tab"
                                    aria-selected=active
                                    aria-label=format!("Show hero slide {}", i + 1)
                                    class="hero-dot"
                                    class:active=active
                                    on:click=move |_| set_slide.set(i)
                                />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            {move || {
                progress.get().map(|p| view! {
                    <section class="continue-reading">
                        <div class="catalog-section-title"><span>{lang.get().text("continue_reading")}</span></div>
                        <div class="book-grid">
                            <div class="book-card">
                                <div class="book-tag">
                                    {format!("Ch. {} — {:.1}%", p.chapter_number, p.completion_percentage)}
                                </div>
                                <h2 class="book-title">{p.book_title.clone()}</h2>
                                <div class="book-author">{p.book_author.clone()}</div>
                                <div class="book-card-footer">
                                    <span>{p.chapter_title.clone()}</span>
                                    <a href=format!("/book/{}", p.book_id) class="btn-read">
                                        <span>{lang.get().text("resume")}</span>
                                        <span>"→"</span>
                                    </a>
                                </div>
                            </div>
                        </div>
                    </section>
                })
            }}

            <main>
                <div class="catalog-section-title">
                    <span>"I"</span>
                    <span>{move || lang.get().text("catalog")}</span>
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
                                    <span>{lang.get().text("read")}</span>
                                    <span>"→"</span>
                                </a>
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
                <div class="load-more">
                    <button class="btn-read" on:click=move |_| load_more() disabled=move || loading.get()>
                        {move || if loading.get() { lang.get().text("loading") } else { lang.get().text("load_more") }}
                    </button>
                </div>
            </main>

            <footer class="footer-vintage">
                <div class="footer-fleuron">"❖"</div>
                <p>{move || if lang.get() == crate::i18n::Lang::Id {
                    "Rotaria — sirkulasi buku domain publik. Kode sumber MIT / Apache-2.0.".to_string()
                } else {
                    "Rotaria — public domain books in circulation. Source MIT / Apache-2.0.".to_string()
                }}</p>
            </footer>
        </div>
    }
}
