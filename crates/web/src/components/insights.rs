//! Quote finder, insight cards, and the spoiler-free recap, rendered as
//! paper cards inside a sheet.

use crate::components::icons;
use crate::components::toast::use_toasts;
use crate::i18n::{use_lang, Lang};
use crate::{api, storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{QuoteSearchRequest, QuoteSearchResultDto};

#[component]
fn Sheet(
    show: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(optional)] wide: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let (lang, _) = use_lang();
    let id = format!("sheet-{}", uuid::Uuid::new_v4().simple());
    let labelled = id.clone();
    view! {
        {move || {
            let id = id.clone();
            let labelled = labelled.clone();
            show.get().then(|| view! {
                <div class="sheet-backdrop" on:click=move |_| show.set(false)>
                    <div class="sheet" class:wide=wide role="dialog" aria-modal="true" aria-labelledby=labelled on:click=|ev| ev.stop_propagation()>
                        <div class="sheet-grip" aria-hidden="true"></div>
                        <div class="sheet-head">
                            <h2 id=id class="section-title">{title.get()}</h2>
                            <button class="btn-icon" aria-label=move || lang.get().text("close") on:click=move |_| show.set(false)>{icons::close()}</button>
                        </div>
                        {children()}
                    </div>
                </div>
            })
        }}
    }
}

fn share_quote(toasts: crate::components::toast::Toasts, text: String, source: String, lang: Lang) {
    spawn_local(async move {
        let body = format!("\u{201C}{text}\u{201D}\n\u{2014} {source}");
        if api::share_text("Rotaria", &body).await {
            return;
        }
        if api::copy_text(&body).await {
            toasts.info(lang.text("quote_saved"));
        } else {
            toasts.error(lang.text("error_generic"));
        }
    });
}

#[component]
pub fn QuoteFinder(book_id: String, show: RwSignal<bool>) -> impl IntoView {
    let (lang, _) = use_lang();
    let toasts = use_toasts();
    let (query, set_query) = signal(String::new());
    let (results, set_results) = signal(Vec::<QuoteSearchResultDto>::new());
    let (busy, set_busy) = signal(false);
    let (searched, set_searched) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (saved, set_saved) = signal(Vec::<uuid::Uuid>::new());
    let book = StoredValue::new(book_id);

    let run = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let q = query.get_untracked();
        if q.trim().len() < 3 {
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        let id = book.get_value();
        spawn_local(async move {
            match api::quote_search(
                &id,
                &QuoteSearchRequest {
                    query: q,
                    limit: Some(5),
                },
            )
            .await
            {
                Ok(items) => set_results.set(items),
                Err(_) => {
                    set_results.set(Vec::new());
                    set_error.set(Some(lang.get_untracked().text("quote_error")));
                }
            }
            set_searched.set(true);
            set_busy.set(false);
        });
    };

    // Members save to the account; guests keep the quote on this device until
    // sign-in merges it (see `AuthSheet`).
    let save = move |row: QuoteSearchResultDto| {
        let id = book.get_value();
        spawn_local(async move {
            let target = match (
                id.parse::<uuid::Uuid>(),
                api::chapter(&id, row.chapter_number).await.map(|c| c.id),
            ) {
                (Ok(book_id), Ok(chapter_id)) => (book_id, chapter_id),
                _ => {
                    toasts.error(lang.get_untracked().text("error_generic"));
                    return;
                }
            };
            let stored = if api::is_authed() {
                api::save_quote(&shared::SaveQuoteRequest {
                    book_id: target.0,
                    chapter_id: target.1,
                    quote_text: row.content.clone(),
                })
                .await
                .map(|_| "quote_saved")
            } else {
                let record = storage::LocalQuoteRecord {
                    key: storage::local_quote_key(target.0, target.1, &row.content),
                    book_id: target.0,
                    chapter_id: target.1,
                    quote_text: row.content.clone(),
                    saved_at: chrono::Utc::now(),
                };
                storage::put(storage::STORE_LOCAL_QUOTES, &record)
                    .await
                    .map(|_| "quote_saved_device")
            };
            match stored {
                Ok(message) => {
                    set_saved.update(|s| s.push(row.chunk_id));
                    toasts.info(lang.get_untracked().text(message));
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
        });
    };

    view! {
        <Sheet show=show title=Signal::derive(move || lang.get().text("quote_finder")) wide=true>
            <form class="search-bar" on:submit=run>
                {icons::search()}
                <label class="visually-hidden" for="quote-query">{move || lang.get().text("quote_finder")}</label>
                <input id="quote-query" type="search" placeholder=move || lang.get().text("quote_ph")
                    prop:value=move || query.get()
                    on:input=move |ev| set_query.set(event_target_value(&ev))/>
                <button type="submit" class="btn btn-primary" style="min-height: 38px; padding: 0 16px" disabled=move || busy.get()>
                    {move || if busy.get() { lang.get().text("loading") } else { lang.get().text("search_cta") }}
                </button>
            </form>
            {move || error.get().map(|e| view! { <p class="form-error" role="alert">{e}</p> })}
            <ol class="quote-results" aria-live="polite">
                {move || (searched.get() && !busy.get() && results.get().is_empty() && error.get().is_none()).then(|| view! {
                    <li class="empty-state"><p>{move || lang.get().text("quote_empty")}</p></li>
                })}
                {move || (!api::is_authed() && !results.get().is_empty()).then(|| view! {
                    <li class="form-hint">{move || lang.get().text("quote_guest_hint")}</li>
                })}
                {move || results.get().into_iter().map(|r| {
                    let id = book.get_value();
                    let jump = format!("/read/{id}?chapter={}", r.chapter_number);
                    let label = format!("{} {} · {:.0}% {}", lang.get().text("chapter"), r.chapter_number, r.similarity_score * 100.0, lang.get().text("similarity"));
                    let is_saved = saved.get().contains(&r.chunk_id);
                    let share_text = r.content.clone();
                    let share_source = format!("{} {}", lang.get().text("chapter"), r.chapter_number);
                    let row_for_save = r.clone();
                    view! {
                        <li class="quote-row">
                            <div class="quote-meta"><span>{label}</span><span>"\u{2756}"</span></div>
                            <blockquote>{r.content.clone()}</blockquote>
                            <div class="quote-actions">
                                <a href=jump class="btn btn-primary">{move || lang.get().text("quote_jump")}</a>
                                <button class="btn btn-ghost" disabled=is_saved on:click=move |_| save(row_for_save.clone())>
                                    {move || if is_saved { lang.get().text("quote_saved") } else { lang.get().text("quote_save") }}
                                </button>
                                <button class="btn btn-ghost" on:click=move |_| share_quote(toasts, share_text.clone(), share_source.clone(), lang.get_untracked())>
                                    {icons::share()}
                                    <span>{move || lang.get().text("quote_share")}</span>
                                </button>
                            </div>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ol>
        </Sheet>
    }
}

fn string_list(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|i| i.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

#[component]
fn InsightList(#[prop(into)] title: String, items: Vec<String>) -> impl IntoView {
    (!items.is_empty()).then(|| {
        view! {
            <section class="insight-card">
                <h4>{title}</h4>
                <ul>{items.into_iter().map(|i| view! { <li>{i}</li> }).collect::<Vec<_>>()}</ul>
            </section>
        }
    })
}

#[component]
fn InsightText(#[prop(into)] title: String, text: Option<String>) -> impl IntoView {
    text.map(|t| {
        view! {
            <section class="insight-card">
                <h4>{title}</h4>
                <p>{t}</p>
            </section>
        }
    })
}

#[component]
pub fn AtomicCards(book_id: String, chapter: i32, show: RwSignal<bool>) -> impl IntoView {
    let (lang, _) = use_lang();
    let (cards, set_cards) = signal(None::<Result<serde_json::Value, ()>>);
    let book = StoredValue::new(book_id);
    // Fetched the first time the sheet opens, not on every page view.
    Effect::new(move || {
        if !show.get() || cards.get_untracked().is_some() {
            return;
        }
        let id = book.get_value();
        spawn_local(async move {
            match api::atomic_cards(&id, &chapter.to_string()).await {
                Ok(dto) => set_cards.set(Some(Ok(dto.cards))),
                Err(_) => set_cards.set(Some(Err(()))),
            }
        });
    });
    view! {
        <Sheet show=show title=Signal::derive(move || format!("{} \u{2014} {} {chapter}", lang.get().text("atomic_cards"), lang.get().text("chapter")))>
            {move || match cards.get() {
                None => view! { <div class="skeleton" style="height: 120px"></div> }.into_any(),
                Some(Err(())) => view! { <p class="empty-state">{lang.get().text("not_generated")}</p> }.into_any(),
                Some(Ok(value)) => view! {
                    <InsightList title=lang.get().text("key_concepts") items=string_list(&value, "key_concepts")/>
                    <InsightList title=lang.get().text("notable_quotes") items=string_list(&value, "notable_quotes")/>
                    <InsightText title=lang.get().text("historical_context") text=string_field(&value, "historical_context")/>
                }.into_any(),
            }}
        </Sheet>
    }
}

#[component]
pub fn CatchupRecap(book_id: String, chapter: i32, show: RwSignal<bool>) -> impl IntoView {
    let (lang, _) = use_lang();
    let (recap, set_recap) = signal(None::<Result<serde_json::Value, ()>>);
    let book = StoredValue::new(book_id);
    Effect::new(move || {
        if !show.get() || recap.get_untracked().is_some() {
            return;
        }
        let id = book.get_value();
        spawn_local(async move {
            match api::chapter_recap(&id, &chapter.to_string()).await {
                Ok(dto) => set_recap.set(Some(Ok(dto.recap))),
                Err(_) => set_recap.set(Some(Err(()))),
            }
        });
    });
    view! {
        <Sheet show=show title=Signal::derive(move || lang.get().text("recap_title"))>
            <p class="form-hint">{move || lang.get().text("recap_note")}</p>
            {move || match recap.get() {
                None => view! { <div class="skeleton" style="height: 120px"></div> }.into_any(),
                Some(Err(())) => view! { <p class="empty-state">{lang.get().text("not_generated")}</p> }.into_any(),
                Some(Ok(value)) => view! {
                    <InsightText title=lang.get().text("recap_title") text=string_field(&value, "summary")/>
                    <InsightList title=lang.get().text("key_characters") items=string_list(&value, "key_characters")/>
                    <InsightText title=lang.get().text("recap_note") text=string_field(&value, "spoiler_free_note")/>
                }.into_any(),
            }}
        </Sheet>
    }
}
