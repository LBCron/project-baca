//! Home: hero with illustration plates, continue reading, search and catalog.

use crate::api;
use crate::components::book_card::{BookCard, SkeletonCard};
use crate::components::icons;
use crate::components::progress::ProgressRing;
use crate::components::session::use_session;
use crate::i18n::use_lang;
use crate::storage;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use shared::{BookCatalogQuery, BookSearchQuery, BookSummaryDto, GuestProgressRecord};
use std::time::Duration;

/// (source, alt, caption) for the rotating hero plates. Captions are Latin
/// plate titles, in keeping with the brand name.
const PLATES: [(&str, &str, &str); 5] = [
    (
        "/assets/library-bookshelf-ladder.png",
        "Reader climbing a library ladder",
        "Plate I \u{2014} Bibliotheca",
    ),
    (
        "/assets/cozy-reader-armchair-owl.png",
        "Reader in an armchair with tea and an owl",
        "Plate II \u{2014} Lectio",
    ),
    (
        "/assets/manuscript-inspection-clothesline.png",
        "Writer drying manuscript pages on a line",
        "Plate III \u{2014} Manuscripta",
    ),
    (
        "/assets/admin-sorting-pigeonholes.png",
        "Archivist sorting mail into pigeonholes",
        "Plate IV \u{2014} Archivum",
    ),
    (
        "/assets/retro-rocket-discovery.png",
        "Victorian explorers in a retro rocket",
        "Plate V \u{2014} Inventio",
    ),
];

const PAGE_SIZE: u64 = 20;
const SHORT_READ_MINUTES: i32 = 120;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Indonesian,
    English,
    Short,
}

impl Filter {
    fn language(self) -> Option<String> {
        match self {
            Filter::Indonesian => Some("id".to_string()),
            Filter::English => Some("en".to_string()),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Resume {
    book_id: String,
    title: String,
    author: String,
    chapter_number: i32,
    chapter_title: String,
    percent: f32,
    cover_url: String,
}

fn search_hit_to_summary(r: shared::BookSearchResultDto) -> BookSummaryDto {
    BookSummaryDto {
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
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let query_map = use_query_map();
    let wants_search = move || query_map.read().get("search").is_some();

    let (books, set_books) = signal(Vec::<BookSummaryDto>::new());
    let (cursor, set_cursor) = signal(None::<String>);
    let (exhausted, set_exhausted) = signal(false);
    let (loading, set_loading) = signal(true);
    let (query, set_query) = signal(String::new());
    let (generation, set_generation) = signal(0u32);
    let (filter, set_filter) = signal(Filter::All);
    let (resume, set_resume) = signal(None::<Resume>);
    let (slide, set_slide) = signal(0usize);
    let (paused, set_paused) = signal(false);

    let load_page = move |reset: bool| {
        let gen = generation.get_untracked() + 1;
        set_generation.set(gen);
        set_loading.set(true);
        if reset {
            set_cursor.set(None);
            set_exhausted.set(false);
        }
        let cur = if reset { None } else { cursor.get_untracked() };
        let language = filter.get_untracked().language();
        spawn_local(async move {
            let page = api::catalog(&BookCatalogQuery {
                cursor: cur.as_deref().and_then(|c| c.parse::<uuid::Uuid>().ok()),
                limit: Some(PAGE_SIZE),
                language,
                ..Default::default()
            })
            .await
            .unwrap_or_default();
            if generation.get_untracked() != gen {
                return;
            }
            set_exhausted.set((page.len() as u64) < PAGE_SIZE);
            if let Some(last) = page.last() {
                set_cursor.set(Some(last.id.to_string()));
            }
            if reset {
                set_books.set(page);
            } else {
                set_books.update(|all| all.extend(page));
            }
            set_loading.set(false);
        });
    };

    let run_search = move |value: String| {
        let gen = generation.get_untracked() + 1;
        set_generation.set(gen);
        set_loading.set(true);
        spawn_local(async move {
            let results = api::search(&BookSearchQuery {
                q: value,
                limit: Some(20),
                ..Default::default()
            })
            .await
            .unwrap_or_default();
            if generation.get_untracked() != gen {
                return;
            }
            set_exhausted.set(true);
            set_books.set(results.into_iter().map(search_hit_to_summary).collect());
            set_loading.set(false);
        });
    };

    let on_search = move |ev| {
        let value = event_target_value(&ev);
        set_query.set(value.clone());
        set_timeout(
            move || {
                if query.get_untracked() != value {
                    return;
                }
                if value.trim().len() < 2 {
                    load_page(true);
                } else {
                    run_search(value.trim().to_string());
                }
            },
            Duration::from_millis(300),
        );
    };

    let pick_filter = move |f: Filter| {
        set_filter.set(f);
        set_query.set(String::new());
        load_page(true);
    };

    load_page(true);

    Effect::new(move || {
        let authed = session.authed.get();
        spawn_local(async move {
            let found = if authed {
                api::active_progress().await.ok().flatten().map(|p| Resume {
                    book_id: p.book_id.to_string(),
                    title: p.book_title,
                    author: p.book_author,
                    chapter_number: p.chapter_number,
                    chapter_title: p.chapter_title,
                    percent: p.completion_percentage,
                    cover_url: p.book_cover_url,
                })
            } else {
                latest_guest_resume().await
            };
            set_resume.set(found);
        });
    });

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
                    set_slide.update(|s| *s = (*s + 1) % PLATES.len());
                }
            },
            Duration::from_millis(7000),
        );
    });

    let visible = move || {
        let all = books.get();
        if filter.get() == Filter::Short {
            all.into_iter()
                .filter(|b| {
                    b.estimated_reading_minutes > 0
                        && b.estimated_reading_minutes <= SHORT_READ_MINUTES
                })
                .collect::<Vec<_>>()
        } else {
            all
        }
    };

    view! {
        <div class="container">
            <section class="hero">
                <div class="hero-copy">
                    <div class="kicker">{move || lang.get().text("hero_sub")}</div>
                    <h1 class="display hero-heading">
                        {move || lang.get().text("hero_head_a")}<em>{move || lang.get().text("hero_head_b")}</em>
                    </h1>
                    <p class="lede">{move || lang.get().text("hero_desc")}</p>
                    <form class="search-bar" role="search" on:submit=|ev| ev.prevent_default()>
                        {icons::search()}
                        <label class="visually-hidden" for="catalog-search">{move || lang.get().text("search_cta")}</label>
                        <input
                            id="catalog-search"
                            type="search"
                            placeholder=move || lang.get().text("search_ph")
                            autofocus=wants_search()
                            prop:value=move || query.get()
                            on:input=on_search
                        />
                    </form>
                    <div class="chip-row" role="group" aria-label=move || lang.get().text("catalog")>
                        <button class="chip" class:active=move || filter.get() == Filter::All aria-pressed=move || filter.get() == Filter::All on:click=move |_| pick_filter(Filter::All)>{move || lang.get().text("chip_all")}</button>
                        <button class="chip" class:active=move || filter.get() == Filter::Indonesian aria-pressed=move || filter.get() == Filter::Indonesian on:click=move |_| pick_filter(Filter::Indonesian)>{move || lang.get().text("chip_id")}</button>
                        <button class="chip" class:active=move || filter.get() == Filter::English aria-pressed=move || filter.get() == Filter::English on:click=move |_| pick_filter(Filter::English)>{move || lang.get().text("chip_en")}</button>
                        <button class="chip" class:active=move || filter.get() == Filter::Short aria-pressed=move || filter.get() == Filter::Short on:click=move |_| pick_filter(Filter::Short)>{move || lang.get().text("chip_short")}</button>
                    </div>
                </div>
                <div
                    class="hero-art"
                    on:mouseenter=move |_| set_paused.set(true)
                    on:mouseleave=move |_| set_paused.set(false)
                    on:focusin=move |_| set_paused.set(true)
                    on:focusout=move |_| set_paused.set(false)
                >
                    <div>
                        <div class="plate">
                            {move || {
                                let (src, alt, caption) = PLATES[slide.get()];
                                view! {
                                    <img src=src alt=alt class="hero-slide"/>
                                    <div class="plate-caption"><span>{caption}</span><span aria-hidden="true">"\u{2756}"</span></div>
                                }
                            }}
                        </div>
                        <div class="hero-dots" role="tablist" aria-label="Hero art">
                            {PLATES.iter().enumerate().map(|(i, _)| {
                                let active = move || slide.get() == i;
                                view! {
                                    <button
                                        role="tab"
                                        aria-selected=active
                                        aria-label=format!("Plate {}", i + 1)
                                        class="hero-dot"
                                        class:active=active
                                        on:click=move |_| set_slide.set(i)
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            </section>

            {move || resume.get().map(|r| {
                let href = format!("/read/{}?chapter={}", r.book_id, r.chapter_number);
                let percent = r.percent;
                view! {
                    <section class="section" style="margin-bottom: 40px">
                        <div class="section-head">
                            <h2 class="section-title">{move || lang.get().text("continue_reading")}</h2>
                        </div>
                        <a href=href class="resume-card">
                            <ProgressRing percent=Signal::derive(move || percent)/>
                            <div class="resume-meta">
                                <small>{move || format!("{} {} \u{00B7} {}", lang.get().text("chapter"), r.chapter_number, r.chapter_title)}</small>
                                <span class="title">{r.title.clone()}</span>
                                <small>{r.author.clone()}</small>
                            </div>
                            <span class="play" aria-hidden="true">{icons::play()}</span>
                        </a>
                    </section>
                }
            })}

            <section id="katalog">
                <div class="section-head">
                    <h2 class="section-title">{move || lang.get().text("catalog")}</h2>
                </div>
                <div class="book-grid">
                    {move || visible().into_iter().map(|b| view! { <BookCard book=b/> }).collect::<Vec<_>>()}
                    {move || (loading.get() && books.get().is_empty()).then(|| (0..8).map(|_| view! { <SkeletonCard/> }).collect::<Vec<_>>())}
                </div>
                {move || (!loading.get() && visible().is_empty()).then(|| view! {
                    <div class="empty-state">
                        <img src="/assets/retro-rocket-discovery.png" alt=""/>
                        <p>{move || lang.get().text("no_results")}</p>
                    </div>
                })}
                {move || (!exhausted.get() && !books.get().is_empty()).then(|| view! {
                    <div style="display: flex; justify-content: center; margin-top: 24px">
                        <button class="btn btn-ghost" on:click=move |_| load_page(false) disabled=move || loading.get()>
                            {move || if loading.get() { lang.get().text("loading") } else { lang.get().text("load_more") }}
                        </button>
                    </div>
                })}
            </section>
        </div>

        <section id="cara-kerja" class="band">
            <div class="container" style="display: flex; flex-direction: column; gap: 28px">
                <div class="fleuron" aria-hidden="true">"\u{2756}"</div>
                <h2 class="display" style="text-align: center; font-size: clamp(1.8rem, 5vw, 2.4rem)">
                    {move || lang.get().text("how_head_a")}<em>{move || lang.get().text("how_head_b")}</em>
                </h2>
                <div class="features">
                    <article class="card feature">
                        <span class="numeral">"I"</span>
                        <h3>{move || lang.get().text("feature_1_t")}</h3>
                        <p>{move || lang.get().text("feature_1_d")}</p>
                    </article>
                    <article class="card feature">
                        <span class="numeral">"II"</span>
                        <h3>{move || lang.get().text("feature_2_t")}</h3>
                        <p>{move || lang.get().text("feature_2_d")}</p>
                    </article>
                    <article class="card feature">
                        <span class="numeral">"III"</span>
                        <h3>{move || lang.get().text("feature_3_t")}</h3>
                        <p>{move || lang.get().text("feature_3_d")}</p>
                    </article>
                </div>
            </div>
        </section>

        <footer class="container site-footer">
            <span>{move || lang.get().text("footer")}</span>
            <span>"MIT / Apache-2.0"</span>
        </footer>
    }
}

/// The most recent guest position on this device, enriched with the book's
/// metadata so the card reads like the signed-in one.
async fn latest_guest_resume() -> Option<Resume> {
    let records = storage::get_all::<GuestProgressRecord>(storage::STORE_GUEST_PROGRESS)
        .await
        .ok()?;
    let latest = records
        .into_iter()
        .max_by_key(|r| r.last_read_at.map(|t| t.timestamp()).unwrap_or(0))?;
    let book = api::book_detail(&latest.book_id.to_string()).await.ok()?;
    let chapter = book
        .chapters
        .iter()
        .find(|c| c.id == latest.last_chapter_id)?;
    Some(Resume {
        book_id: book.id.to_string(),
        title: book.title,
        author: book.author,
        chapter_number: chapter.chapter_number,
        chapter_title: chapter.title.clone(),
        percent: latest.completion_percentage,
        cover_url: book.cover_url,
    })
}
