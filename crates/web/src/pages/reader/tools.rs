//! Reader sheets that never leave the book: the chapter drawer and the
//! search through the book's text.

use crate::api;
use crate::components::icons;
use crate::components::insights::Sheet;
use crate::format::roman;
use crate::i18n::use_lang;
use crate::storage::{self, OfflineChapterRecord};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{text_hit_snippet, BookDetailDto, BookTextHitDto, BOOK_TEXT_HITS_MAX};
use wasm_bindgen::JsCast;

/// Every chapter with where the reader stands: read, here, or ahead.
#[component]
pub fn ChapterDrawer(
    book: ReadSignal<Option<BookDetailDto>>,
    current: ReadSignal<i32>,
    show: RwSignal<bool>,
    on_pick: Callback<i32>,
) -> impl IntoView {
    let (lang, _) = use_lang();
    view! {
        <Sheet show=show title=Signal::derive(move || lang.get().text("chapters"))>
            <ol class="chapter-list chapter-drawer">
                {move || book.get().map(|b| b.chapters).unwrap_or_default().into_iter().map(|c| {
                    let number = c.chapter_number;
                    let here = move || current.get() == number;
                    let read = move || number < current.get();
                    view! {
                        <li class="chapter-row" class:here=here class:read=read>
                            <button aria-current=move || here().then_some("true") on:click=move |_| on_pick.run(number)>
                                <span class="numeral">{roman(number)}</span>
                                <span class="label">{c.title.clone()}</span>
                                <span class="meta">
                                    {move || if here() {
                                        lang.get().text("chapter_here")
                                    } else if read() {
                                        lang.get().text("chapter_read")
                                    } else {
                                        String::new()
                                    }}
                                </span>
                            </button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ol>
        </Sheet>
    }
}

/// Paragraph hits from the chapters saved on this device, read the way the
/// reader numbers them (`p-N`).
async fn search_on_device(book: &str, query: &str) -> Vec<BookTextHitDto> {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    let mut chapters: Vec<OfflineChapterRecord> =
        storage::get_all::<OfflineChapterRecord>(storage::STORE_OFFLINE_CHAPTERS)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.book_id.to_string() == book)
            .collect();
    chapters.sort_by_key(|c| c.chapter_number);
    let mut hits = Vec::new();
    for chapter in chapters {
        // A detached element parses the stored (already sanitized) HTML
        // without running or fetching anything.
        let Ok(holder) = document.create_element("div") else {
            continue;
        };
        // A line break reads as a space, as the server's paragraph index
        // reads it; textContent alone would run verse lines together.
        holder.set_inner_html(&chapter.html_content.replace("<br/>", "<br/> "));
        let Ok(paragraphs) = holder.query_selector_all("p") else {
            continue;
        };
        for index in 0..paragraphs.length() {
            let text = paragraphs
                .item(index)
                .and_then(|n| n.text_content())
                .unwrap_or_default();
            if let Some(snippet) = text_hit_snippet(&text, query) {
                hits.push(BookTextHitDto {
                    chapter_number: chapter.chapter_number,
                    chapter_title: chapter.title.clone(),
                    paragraph: index as i32 + 1,
                    snippet,
                });
                if hits.len() == BOOK_TEXT_HITS_MAX {
                    return hits;
                }
            }
        }
    }
    hits
}

/// Byte range of the searched phrase in a snippet, folded per character
/// as the search matched it, so positions carry over to the original text.
fn mark_range(snippet: &str, query: &str) -> Option<(usize, usize)> {
    let hay: Vec<char> = snippet.chars().map(shared::fold).collect();
    let needle: Vec<char> = shared::fold_for_search(query).chars().collect();
    if needle.is_empty() {
        return None;
    }
    let start = hay
        .windows(needle.len())
        .position(|w| w == needle.as_slice())?;
    let byte = |chars: usize| {
        snippet
            .char_indices()
            .nth(chars)
            .map_or(snippet.len(), |(i, _)| i)
    };
    Some((byte(start), byte(start + needle.len())))
}

/// The snippet with the searched phrase marked.
fn marked(snippet: &str, query: &str) -> impl IntoView {
    match mark_range(snippet, query) {
        Some((at, end)) => view! {
            <span>{snippet[..at].to_string()}</span>
            <mark>{snippet[at..end].to_string()}</mark>
            <span>{snippet[end..].to_string()}</span>
        }
        .into_any(),
        None => view! { <span>{snippet.to_string()}</span> }.into_any(),
    }
}

#[derive(Clone)]
struct Found {
    query: String,
    hits: Vec<BookTextHitDto>,
    on_device: bool,
}

/// Search through the book's text: the server when online, the copy saved
/// on this device otherwise. Picking a hit opens the page that holds it.
#[component]
pub fn BookSearch(
    book_id: String,
    show: RwSignal<bool>,
    on_pick: Callback<(i32, i32)>,
) -> impl IntoView {
    let (lang, _) = use_lang();
    let book = StoredValue::new(book_id);
    let (query, set_query) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (found, set_found) = signal(None::<Found>);

    let run = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let q = query.get_untracked().trim().to_string();
        if q.chars().count() < 2 {
            return;
        }
        set_busy.set(true);
        let id = book.get_value();
        spawn_local(async move {
            let (hits, on_device) = match api::search_book_text(&id, &q).await {
                Ok(hits) => (hits, false),
                Err(_) => (search_on_device(&id, &q).await, true),
            };
            set_found.try_set(Some(Found {
                query: q,
                hits,
                on_device,
            }));
            set_busy.try_set(false);
        });
    };

    view! {
        <Sheet show=show title=Signal::derive(move || lang.get().text("book_search")) wide=true>
            <form class="search-bar" role="search" on:submit=run>
                {icons::search()}
                <label class="visually-hidden" for="book-search">{move || lang.get().text("book_search")}</label>
                <input
                    id="book-search"
                    type="search"
                    minlength="2"
                    maxlength="200"
                    data-autofocus=""
                    placeholder=move || lang.get().text("book_search_ph")
                    prop:value=move || query.get()
                    on:input=move |ev| set_query.set(event_target_value(&ev))
                />
                <button type="submit" class="btn btn-primary search-submit" disabled=move || busy.get()>
                    {move || lang.get().text("search_cta")}
                </button>
            </form>
            {move || found.get().map(|f| {
                let note = f.on_device.then(|| view! { <p class="form-hint">{lang.get().text("book_search_offline")}</p> });
                if f.hits.is_empty() {
                    return view! { {note} <p class="empty-state">{lang.get().text("book_search_none")}</p> }.into_any();
                }
                let q = f.query.clone();
                view! {
                    {note}
                    <ol class="search-hits">
                        {f.hits.into_iter().map(|hit| {
                            let target = (hit.chapter_number, hit.paragraph);
                            view! {
                                <li>
                                    <button class="search-hit" data-chapter=hit.chapter_number data-paragraph=hit.paragraph on:click=move |_| on_pick.run(target)>
                                        <span class="hit-where">{format!("{} {} \u{00B7} {}", lang.get_untracked().text("chapter"), roman(hit.chapter_number), hit.chapter_title)}</span>
                                        <span class="hit-text">{marked(&hit.snippet, &q)}</span>
                                    </button>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ol>
                }.into_any()
            })}
        </Sheet>
    }
}

/// Text of the current selection when it lies inside the chapter body, so
/// selecting in the bar or a sheet never offers to save it.
pub fn selected_chapter_text() -> Option<String> {
    let selection = web_sys::window()?.get_selection().ok()??;
    if selection.is_collapsed() {
        return None;
    }
    // Both ends: a drag past the body would pick up a folio's page number,
    // which the server then refuses as not in the chapter.
    let in_body = |node: Option<web_sys::Node>| {
        let node = node?;
        let element = match node.dyn_ref::<web_sys::Element>() {
            Some(el) => el.clone(),
            None => node.parent_element()?,
        };
        element.closest(".chapter-body").ok()?
    };
    in_body(selection.anchor_node())?;
    in_body(selection.focus_node())?;
    let text: String = selection.to_string().into();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty() && text.chars().count() <= 2000).then_some(text)
}

/// Calls `on_change` on every selection change until the reader unmounts.
/// `selectionchange` fires on the document only, never on the window.
pub fn watch_selection(on_change: impl Fn() + 'static) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let listener = wasm_bindgen::closure::Closure::<dyn Fn()>::new(on_change);
    if document
        .add_event_listener_with_callback("selectionchange", listener.as_ref().unchecked_ref())
        .is_err()
    {
        return;
    }
    let attached = send_wrapper::SendWrapper::new((document, listener));
    on_cleanup(move || {
        let (document, listener) = attached.take();
        let _ = document.remove_event_listener_with_callback(
            "selectionchange",
            listener.as_ref().unchecked_ref(),
        );
    });
}

pub fn clear_selection() {
    if let Some(selection) = web_sys::window().and_then(|w| w.get_selection().ok().flatten()) {
        let _ = selection.remove_all_ranges();
    }
}

#[cfg(test)]
mod tests {
    use super::mark_range;

    #[test]
    fn the_mark_follows_the_search_folding() {
        let snippet = "Di \u{0130}STANBUL kapal tiba";
        let (at, end) = mark_range(snippet, "istanbul").expect("found");
        assert_eq!(&snippet[at..end], "\u{0130}STANBUL");
        let spaced = "the secure ancestral hall";
        let (at, end) = mark_range(spaced, "secure   ancestral").expect("found");
        assert_eq!(&spaced[at..end], "secure ancestral");
        assert_eq!(mark_range("nothing here", "absent"), None);
        assert_eq!(mark_range("anything", "   "), None);
    }
}
