//! Home: a greeting and the current book on phones, an editorial hero on
//! wide screens; then search, quick filters, the catalog, and the
//! "how it works" band.

use crate::api;
use crate::components::anchor::{reveal, scroll_to_location_hash};
use crate::components::book_card::{BookCard, Cover, SkeletonCard};
use crate::components::icons;
use crate::components::progress::{ProgressBar, ProgressRing};
use crate::components::session::use_session;
use crate::format::roman;
use crate::i18n::use_lang;
use crate::storage;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use shared::{BookCatalogQuery, BookSearchQuery, BookSummaryDto, GuestProgressRecord};
use std::time::Duration;

const PLATE_SRC: &str = "/assets/library-bookshelf-ladder.webp";
const PAGE_SIZE: u64 = 20;
/// Rows of covers shown before "see the whole catalog".
const CATALOG_PREVIEW_ROWS: usize = 2;
const SHORT_READ_MINUTES: i32 = 120;
const RAIL_SIZE: usize = 8;
const THEME_CHIP_LIMIT: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LanguageFilter {
    All,
    Indonesian,
    English,
}

impl LanguageFilter {
    fn code(self) -> Option<String> {
        match self {
            LanguageFilter::Indonesian => Some("id".to_string()),
            LanguageFilter::English => Some("en".to_string()),
            LanguageFilter::All => None,
        }
    }
}

/// Hero chip filters, applied on the client over the loaded page.
#[derive(Clone, PartialEq, Eq)]
enum Quick {
    Theme(String),
    Short,
}

impl Quick {
    fn keeps(&self, book: &BookSummaryDto) -> bool {
        match self {
            Quick::Theme(theme) => book.primary_theme.eq_ignore_ascii_case(theme),
            Quick::Short => {
                book.estimated_reading_minutes > 0
                    && book.estimated_reading_minutes <= SHORT_READ_MINUTES
            }
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

fn greeting_key(hour: u32) -> &'static str {
    match hour {
        4..=10 => "greeting_morning",
        11..=14 => "greeting_afternoon",
        15..=17 => "greeting_evening",
        _ => "greeting_night",
    }
}

/// Distinct themes of the loaded books, in catalog order.
fn theme_chips(books: &[BookSummaryDto]) -> Vec<String> {
    let mut themes: Vec<String> = Vec::new();
    for book in books {
        let theme = book.primary_theme.trim();
        if theme.is_empty() || themes.iter().any(|t| t.eq_ignore_ascii_case(theme)) {
            continue;
        }
        themes.push(theme.to_string());
        if themes.len() == THEME_CHIP_LIMIT {
            break;
        }
    }
    themes
}

/// Cover columns at a viewport width, mirroring the grid breakpoints.
fn catalog_columns(width: f64) -> usize {
    if width >= 1440.0 {
        5
    } else if width >= 1024.0 {
        4
    } else if width >= 640.0 {
        3
    } else {
        2
    }
}

fn window_width() -> f64 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(1280.0)
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

/// The current book: a compact row with a progress ring on phones, a wide
/// card with the cover, a progress bar, and two actions on larger screens.
#[component]
fn ResumeCard(resume: Resume, wide: bool) -> impl IntoView {
    let (lang, _) = use_lang();
    let read_href = format!("/read/{}?chapter={}", resume.book_id, resume.chapter_number);
    let recap_href = format!("{read_href}&recap=1");
    let percent = resume.percent;
    let chapter_no = resume.chapter_number;
    let chapter_title = resume.chapter_title.clone();
    let title = resume.title.clone();
    let author = resume.author.clone();
    let cover_url = resume.cover_url.clone();
    let chapter_line = move || {
        format!(
            "{} {} \u{00B7} {}",
            lang.get().text("chapter"),
            roman(chapter_no),
            chapter_title
        )
    };
    if wide {
        view! {
            <article class="resume-card resume-wide">
                <div class="resume-thumb"><Cover title=title.clone() cover_url=cover_url/></div>
                <div class="resume-meta">
                    <small>{chapter_line}</small>
                    <h3 class="title">{title}</h3>
                    <small>{move || format!("{author} \u{00B7} {}", lang.get().text_with("percent_done", "n", &format!("{percent:.0}")))}</small>
                    <ProgressBar percent=Signal::derive(move || percent) label=Signal::derive(move || lang.get().text("reading_progress"))/>
                </div>
                <div class="resume-actions">
                    <a href=read_href class="btn btn-primary">{move || lang.get().text("resume")}</a>
                    {(chapter_no > 1).then(move || view! {
                        <a href=recap_href class="btn btn-ghost">{move || lang.get().text("recap_last")}</a>
                    })}
                </div>
            </article>
        }
        .into_any()
    } else {
        view! {
            <a href=read_href class="resume-card">
                <ProgressRing percent=Signal::derive(move || percent)/>
                <div class="resume-meta">
                    <small>{chapter_line}</small>
                    <span class="title">{title}</span>
                    <small>{author}</small>
                </div>
                <span class="play" aria-hidden="true">{icons::play()}</span>
            </a>
        }
        .into_any()
    }
}

#[component]
fn RailCard(book: BookSummaryDto) -> impl IntoView {
    let (lang, _) = use_lang();
    let href = format!("/book/{}", book.id);
    let minutes = book.estimated_reading_minutes;
    let language = book.language.to_uppercase();
    view! {
        <a href=href class="rail-card">
            <Cover title=book.title.clone() cover_url=book.cover_url.clone()/>
            <span class="rail-title">{book.title.clone()}</span>
            <span class="rail-meta">
                {move || if minutes > 0 { format!("{language} \u{00B7} {}", lang.get().duration(minutes)) } else { language.clone() }}
            </span>
        </a>
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
    let (load_error, set_load_error) = signal(false);
    let (show_all, set_show_all) = signal(false);
    let (viewport_width, set_viewport_width) = signal(window_width());
    let (query, set_query) = signal(String::new());
    let (generation, set_generation) = signal(0u32);
    let (filter, set_filter) = signal(LanguageFilter::All);
    let (quick, set_quick) = signal(None::<Quick>);
    let (resume, set_resume) = signal(None::<Resume>);
    let hour = js_sys::Date::new_0().get_hours();
    let search_ref = NodeRef::<leptos::html::Input>::new();
    let first_load = StoredValue::new(true);

    // The search tab lands here with `?search=1`; `autofocus` only works
    // once per document, so focus and reveal the field explicitly.
    Effect::new(move || {
        if !wants_search() {
            return;
        }
        if let Some(input) = search_ref.get() {
            let _ = input.focus();
            reveal(&input);
        }
    });

    let load_page = move |reset: bool| {
        let gen = generation.get_untracked() + 1;
        set_generation.set(gen);
        set_loading.set(true);
        if reset {
            set_cursor.set(None);
            set_exhausted.set(false);
        }
        let cur = if reset { None } else { cursor.get_untracked() };
        let language = filter.get_untracked().code();
        spawn_local(async move {
            let outcome = api::catalog(&BookCatalogQuery {
                cursor: cur.as_deref().and_then(|c| c.parse::<uuid::Uuid>().ok()),
                limit: Some(PAGE_SIZE),
                language,
                ..Default::default()
            })
            .await;
            if generation.try_get_untracked() != Some(gen) {
                return;
            }
            let page = match outcome {
                Ok(page) => page,
                Err(_) => {
                    set_load_error.set(true);
                    set_loading.set(false);
                    return;
                }
            };
            set_load_error.set(false);
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
            if first_load.try_get_value() == Some(true) {
                first_load.set_value(false);
                request_animation_frame(scroll_to_location_hash);
            }
        });
    };

    let run_search = move |value: String| {
        let gen = generation.get_untracked() + 1;
        set_generation.set(gen);
        set_loading.set(true);
        spawn_local(async move {
            let outcome = api::search(&BookSearchQuery {
                q: value,
                limit: Some(20),
                ..Default::default()
            })
            .await;
            if generation.try_get_untracked() != Some(gen) {
                return;
            }
            let results = match outcome {
                Ok(results) => results,
                Err(_) => {
                    set_load_error.set(true);
                    set_loading.set(false);
                    return;
                }
            };
            set_load_error.set(false);
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
                if query.try_get_untracked() != Some(value.clone()) {
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

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let value = query.get_untracked().trim().to_string();
        if value.len() >= 2 {
            run_search(value);
        }
    };

    let resized = window_event_listener(leptos::ev::resize, move |_| {
        set_viewport_width.set(window_width());
    });
    on_cleanup(move || resized.remove());

    let back_online = window_event_listener(leptos::ev::online, move |_| {
        if load_error.try_get_untracked() == Some(true) {
            load_page(true);
        }
    });
    on_cleanup(move || back_online.remove());

    let pick_filter = move |f: LanguageFilter| {
        set_filter.set(f);
        set_query.set(String::new());
        load_page(true);
    };

    let pick_quick = move |q: Quick| {
        set_quick.update(|current| {
            *current = if current.as_ref() == Some(&q) {
                None
            } else {
                Some(q)
            }
        });
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

    let visible = move || {
        let all = books.get();
        match quick.get() {
            Some(q) => all.into_iter().filter(|b| q.keeps(b)).collect::<Vec<_>>(),
            None => all,
        }
    };
    // The home catalog is a two-row preview; searching, filtering, or asking
    // for the whole catalog shows every loaded book with paging.
    let browsing = move || {
        show_all.get()
            || quick.get().is_some()
            || !query.get().trim().is_empty()
            || filter.get() != LanguageFilter::All
    };
    let preview_cap = move || CATALOG_PREVIEW_ROWS * catalog_columns(viewport_width.get());
    let shown = move || {
        let all = visible();
        if browsing() {
            all
        } else {
            all.into_iter().take(preview_cap()).collect()
        }
    };
    let preview_truncated =
        move || !browsing() && (visible().len() > preview_cap() || !exhausted.get());
    let chips = move || theme_chips(&books.get());
    let headline = move || {
        if resume.get().is_some() {
            ("headline_resume_a", "headline_resume_b")
        } else {
            ("headline_fresh_a", "headline_fresh_b")
        }
    };
    let short_active = move || quick.get() == Some(Quick::Short);
    let language_chip = move |f: LanguageFilter, key: &'static str| {
        let active = move || filter.get() == f;
        view! {
            <button class="chip" class:active=active aria-pressed=active on:click=move |_| pick_filter(f)>
                {move || lang.get().text(key)}
            </button>
        }
    };

    view! {
        <div class="container">
            <section class="hero">
                <div class="hero-copy">
                    <div class="hero-pitch wide-only">
                        <div class="kicker">{move || lang.get().text("hero_sub")}</div>
                        <h1 class="display hero-heading">
                            {move || lang.get().text("hero_head_a")}<em>{move || lang.get().text("hero_head_b")}</em>
                        </h1>
                        <p class="lede">{move || lang.get().text("hero_desc")}</p>
                    </div>
                    <div class="greeting narrow-only">
                        <span class="greeting-time">{move || lang.get().text(greeting_key(hour))}</span>
                        <h1 class="display hero-heading">
                            {move || lang.get().text(headline().0)}<em>{move || lang.get().text(headline().1)}</em>
                        </h1>
                    </div>
                    {move || resume.get().map(|r| view! {
                        <div class="narrow-only"><ResumeCard resume=r wide=false/></div>
                    })}
                    <form class="search-bar" role="search" on:submit=on_submit>
                        {icons::search()}
                        <label class="visually-hidden" for="catalog-search">{move || lang.get().text("search_cta")}</label>
                        <input
                            id="catalog-search"
                            type="search"
                            placeholder=move || lang.get().text("search_ph")
                            node_ref=search_ref
                            prop:value=move || query.get()
                            on:input=on_search
                        />
                        <button type="submit" class="btn btn-primary search-submit">{move || lang.get().text("search_cta")}</button>
                    </form>
                    <div class="chip-row" role="group" aria-label=move || lang.get().text("theme_filter")>
                        {move || chips().into_iter().map(|theme| {
                            let q = Quick::Theme(theme.clone());
                            let active = {
                                let q = q.clone();
                                move || quick.get().as_ref() == Some(&q)
                            };
                            view! {
                                <button class="chip" class:active=active.clone() aria-pressed=active on:click=move |_| pick_quick(q.clone())>{theme}</button>
                            }
                        }).collect::<Vec<_>>()}
                        <button class="chip" class:active=short_active aria-pressed=short_active on:click=move |_| pick_quick(Quick::Short)>
                            {move || lang.get().text("chip_short")}
                        </button>
                    </div>
                    {move || (!books.get().is_empty()).then(|| view! {
                        <section class="picks narrow-only" aria-labelledby="picks-title">
                            <div class="section-head">
                                <h2 id="picks-title" class="section-title">{move || lang.get().text("picks")}</h2>
                                <a href="#katalog" class="section-link">{move || lang.get().text("see_all")}</a>
                            </div>
                            <div class="book-rail">
                                {books.get().into_iter().take(RAIL_SIZE).map(|b| view! { <RailCard book=b/> }).collect::<Vec<_>>()}
                            </div>
                        </section>
                    })}
                </div>
                <div class="hero-art wide-only">
                    <figure class="plate">
                        <img src=PLATE_SRC alt=move || lang.get().text("plate_alt") class="hero-slide" width="640" height="640"/>
                        <figcaption class="plate-caption">
                            <span>{move || lang.get().text("plate_caption")}</span>
                            <span aria-hidden="true">"\u{2756}"</span>
                        </figcaption>
                    </figure>
                </div>
            </section>

            {move || resume.get().map(|r| view! {
                <section class="section wide-only" aria-labelledby="resume-title">
                    <div class="section-head">
                        <h2 id="resume-title" class="section-title">{move || lang.get().text("continue_reading")}</h2>
                        <a href="/me" class="section-link">{move || lang.get().text("see_shelf")}</a>
                    </div>
                    <ResumeCard resume=r wide=true/>
                </section>
            })}

            <section id="katalog" aria-labelledby="catalog-title">
                <div class="section-head catalog-head">
                    <h2 id="catalog-title" class="section-title">{move || lang.get().text("catalog")}</h2>
                    <div class="chip-row" role="group" aria-label=move || lang.get().text("language")>
                        {language_chip(LanguageFilter::All, "chip_all")}
                        {language_chip(LanguageFilter::Indonesian, "chip_id")}
                        {language_chip(LanguageFilter::English, "chip_en")}
                    </div>
                </div>
                <div class="book-grid">
                    {move || shown().into_iter().map(|b| view! { <BookCard book=b/> }).collect::<Vec<_>>()}
                    {move || (loading.get() && books.get().is_empty()).then(|| (0..8).map(|_| view! { <SkeletonCard/> }).collect::<Vec<_>>())}
                </div>
                {move || load_error.get().then(|| view! {
                    <div class="empty-state" role="alert">
                        <p>{move || lang.get().text(if api::is_online() { "load_failed" } else { "offline_notice" })}</p>
                        <button class="btn btn-primary" on:click=move |_| load_page(true)>{move || lang.get().text("retry")}</button>
                    </div>
                })}
                {move || (!loading.get() && !load_error.get() && visible().is_empty()).then(|| view! {
                    <div class="empty-state">
                        <img src="/assets/retro-rocket-discovery.webp" alt="" width="768" height="512"/>
                        <p>{move || lang.get().text("no_results")}</p>
                    </div>
                })}
                {move || preview_truncated().then(|| view! {
                    <div class="load-more">
                        <button class="btn btn-ghost" on:click=move |_| set_show_all.set(true)>
                            <span>{move || lang.get().text("see_all_catalog")}</span>
                            {icons::arrow_right()}
                        </button>
                    </div>
                })}
                {move || (browsing() && !exhausted.get() && !books.get().is_empty()).then(|| view! {
                    <div class="load-more">
                        <button class="btn btn-ghost" on:click=move |_| load_page(false) disabled=move || loading.get()>
                            {move || if loading.get() { lang.get().text("loading") } else { lang.get().text("load_more") }}
                        </button>
                    </div>
                })}
            </section>
        </div>

        <section id="cara-kerja" class="band">
            <div class="container band-inner">
                <div class="fleuron" aria-hidden="true">"\u{2756}"</div>
                <h2 class="display band-title">
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
        .filter(|r| !r.is_finished.unwrap_or(false))
        .max_by_key(|r| r.last_read_at)?;
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
