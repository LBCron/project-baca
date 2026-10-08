//! Book overview: cover, synopsis, chapter list, offline save, share, and
//! the action dock that leads into the reader and the insight sheets.

use crate::components::book_card::Cover;
use crate::components::icons;
use crate::components::insights::{AtomicCards, QuoteFinder};
use crate::components::session::use_session;
use crate::components::toast::use_toasts;
use crate::format::{reading_minutes, roman};
use crate::i18n::use_lang;
use crate::{api, storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared::{BookDetailDto, GuestProgressRecord};
use storage::OfflineChapterRecord;

/// Last chapter the reader reached in this book, from the account or the
/// device, so the primary action can say "Continue" instead of "Start".
async fn resume_chapter(book: &BookDetailDto, authed: bool) -> Option<i32> {
    let chapter_id = if authed {
        api::active_progress()
            .await
            .ok()
            .flatten()
            .filter(|p| p.book_id == book.id)
            .map(|p| p.chapter_id)
    } else {
        storage::get_all::<GuestProgressRecord>(storage::STORE_GUEST_PROGRESS)
            .await
            .ok()?
            .into_iter()
            .find(|r| r.book_id == book.id)
            .map(|r| r.last_chapter_id)
    }?;
    book.chapters
        .iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.chapter_number)
}

fn current_url() -> String {
    web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_default()
}

#[component]
pub fn BookPage() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let params = use_params_map();
    let book_id = move || params.get().get("id").unwrap_or_default();
    let (book, set_book) = signal(None::<BookDetailDto>);
    let (error, set_error) = signal(None::<String>);
    let (saving, set_saving) = signal(false);
    let (saved, set_saved) = signal(false);
    let (resume_at, set_resume_at) = signal(None::<i32>);
    let quotes_open = RwSignal::new(false);
    let cards_open = RwSignal::new(false);

    let load = move || {
        let id = book_id();
        set_error.set(None);
        spawn_local(async move {
            match api::book_detail(&id).await {
                Ok(detail) => {
                    let offline = storage::get_all::<BookDetailDto>(storage::STORE_OFFLINE_BOOKS)
                        .await
                        .unwrap_or_default();
                    set_saved.set(offline.iter().any(|b| b.id == detail.id));
                    set_resume_at
                        .set(resume_chapter(&detail, session.authed.get_untracked()).await);
                    set_book.set(Some(detail));
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };
    load();

    let save_offline = move |_| {
        let id = book_id();
        set_saving.set(true);
        spawn_local(async move {
            let outcome = async {
                let bundle = api::offline_bundle(&id).await?;
                storage::put(storage::STORE_OFFLINE_BOOKS, &bundle.book).await?;
                for chapter in &bundle.chapters {
                    let record = OfflineChapterRecord {
                        chapter_id: chapter.id,
                        book_id: chapter.book_id,
                        html_content: chapter.html_content.clone(),
                        chapter_number: chapter.chapter_number,
                        title: chapter.title.clone(),
                    };
                    storage::put(storage::STORE_OFFLINE_CHAPTERS, &record).await?;
                }
                Ok::<(), String>(())
            }
            .await;
            match outcome {
                Ok(()) => {
                    set_saved.set(true);
                    toasts.info(lang.get_untracked().text("saved_offline"));
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
            set_saving.set(false);
        });
    };

    let share = move |_| {
        let title = book
            .get_untracked()
            .map(|b| b.title)
            .unwrap_or_else(|| "Rotaria".to_string());
        spawn_local(async move {
            let url = current_url();
            if api::share_text(&title, &url).await {
                return;
            }
            if api::copy_text(&url).await {
                toasts.info(lang.get_untracked().text("share_copied"));
            } else {
                toasts.error(lang.get_untracked().text("error_generic"));
            }
        });
    };

    let save_label = move || {
        if saved.get() {
            lang.get().text("saved_offline")
        } else {
            lang.get().text("save_offline")
        }
    };

    view! {
        <div class="container">
            {move || match book.get() {
                None => match error.get() {
                    None => view! {
                        <div class="book-overview" aria-busy="true">
                            <div class="book-hero">
                                <div class="skeleton cover-skeleton"></div>
                                <div class="book-hero-meta">
                                    <div class="skeleton skeleton-line" style="width: 40%"></div>
                                    <div class="skeleton skeleton-line" style="width: 70%; height: 2em"></div>
                                    <div class="skeleton skeleton-line" style="width: 50%"></div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    Some(err) => {
                        let not_found = api::is_not_found(&err);
                        let message = if not_found {
                            "not_found"
                        } else if api::is_online() {
                            "book_load_failed"
                        } else {
                            "offline_notice"
                        };
                        view! {
                            <div class="empty-state" role=if not_found { "status" } else { "alert" }>
                                <p>{move || lang.get().text(message)}</p>
                                <div class="sheet-actions centered">
                                    {(!not_found).then(|| view! {
                                        <button class="btn btn-primary" on:click=move |_| load()>{move || lang.get().text("retry")}</button>
                                    })}
                                    <a href="/" class="btn btn-ghost">{move || lang.get().text("back_to_catalog")}</a>
                                </div>
                            </div>
                        }.into_any()
                    }
                },
                Some(b) => {
                    let total_minutes = if b.estimated_reading_minutes > 0 { b.estimated_reading_minutes } else { reading_minutes(b.total_words) };
                    let first_chapter = b.chapters.first().map(|c| c.chapter_number).unwrap_or(1);
                    let target = resume_at.get().unwrap_or(first_chapter);
                    let read_href = format!("/read/{}?chapter={}", b.id, target);
                    let chapter_count = b.chapters.len();
                    let year = b.publication_year.map(|y| format!(" \u{00B7} {y}")).unwrap_or_default();
                    view! {
                        <section class="book-overview">
                            <div class="page-tools">
                                <a href="/" class="btn-icon plain" aria-label=move || lang.get().text("back_to_catalog")>{icons::back()}</a>
                                <div class="page-tools-group">
                                    <button
                                        class="btn-icon plain"
                                        aria-label=save_label
                                        aria-pressed=move || saved.get()
                                        disabled=move || saving.get() || saved.get()
                                        on:click=save_offline
                                    >
                                        {move || if saved.get() { icons::check().into_any() } else { icons::download().into_any() }}
                                    </button>
                                    <button class="btn-icon plain" aria-label=move || lang.get().text("share_book") on:click=share>
                                        {icons::share()}
                                    </button>
                                </div>
                            </div>
                            <div class="book-hero">
                                <Cover title=b.title.clone() cover_url=b.cover_url.clone()/>
                                <div class="book-hero-meta">
                                    <div class="kicker">{b.primary_theme.clone()}</div>
                                    <h1 class="display">{b.title.clone()}</h1>
                                    <p class="book-author">{format!("{}{year}", b.author)}</p>
                                    <div class="meta-row">
                                        <span>{move || lang.get().duration(total_minutes)}</span>
                                        <span>{move || lang.get().text_with("chapters_count", "n", &chapter_count.to_string())}</span>
                                        <span>{b.language.to_uppercase()}</span>
                                    </div>
                                </div>
                            </div>
                            {(!b.description.trim().is_empty()).then(|| view! { <p class="prose">{b.description.clone()}</p> })}
                            {(!b.tags.is_empty()).then(|| view! {
                                <div class="chip-row">{b.tags.iter().map(|t| view! { <span class="tag">{format!("#{}", t.replace(' ', ""))}</span> }).collect::<Vec<_>>()}</div>
                            })}
                            <div class="book-dock">
                                <a href=read_href class="btn btn-primary btn-lg">
                                    <span>{move || match resume_at.get() {
                                        Some(n) => format!("{} {}", lang.get().text("continue_chapter"), roman(n)),
                                        None => lang.get().text("start_reading"),
                                    }}</span>
                                    {icons::arrow_right()}
                                </a>
                                <div class="book-actions-row">
                                    <button class="btn btn-ghost" on:click=move |_| quotes_open.set(true)>{icons::search()}<span>{move || lang.get().text("quote_finder")}</span></button>
                                    <button class="btn btn-ghost" on:click=move |_| cards_open.set(true)>{icons::cards()}<span>{move || lang.get().text("atomic_cards")}</span></button>
                                </div>
                            </div>
                            <QuoteFinder book_id=b.id.to_string() show=quotes_open/>
                            <AtomicCards book_id=b.id.to_string() chapter=target show=cards_open/>
                            <div class="chapters">
                                <div class="section-head">
                                    <h2 class="section-title">{move || lang.get().text("chapters")}</h2>
                                    {move || resume_at.get().map(|n| view! { <span class="form-hint">{format!("{}: {} {}", lang.get().text("last_read"), lang.get().text("chapter"), roman(n))}</span> })}
                                </div>
                                <ol class="chapter-list">
                                    {b.chapters.iter().map(|c| {
                                        let href = format!("/read/{}?chapter={}", b.id, c.chapter_number);
                                        let number = c.chapter_number;
                                        let minutes = reading_minutes(c.word_count);
                                        let is_current = move || resume_at.get() == Some(number);
                                        let is_done = move || resume_at.get().map(|n| number < n).unwrap_or(false);
                                        view! {
                                            <li class="chapter-row" class:current=is_current class:done=is_done>
                                                <a href=href>
                                                    <span class="numeral">{roman(number)}</span>
                                                    <span class="label">{c.title.clone()}</span>
                                                    <span class="meta">
                                                        {move || if is_done() { icons::check().into_any() } else { lang.get().duration(minutes).into_any() }}
                                                    </span>
                                                </a>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()}
                                </ol>
                            </div>
                        </section>
                    }.into_any()
                }
            }}
        </div>
    }
}
