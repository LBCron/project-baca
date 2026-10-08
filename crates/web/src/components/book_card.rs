//! Catalog card and the typographic cover used when a book has no image.

use crate::api;
use crate::components::icons;
use crate::i18n::use_lang;
use leptos::prelude::*;
use shared::BookSummaryDto;

const COVER_TINTS: [&str; 6] = [
    "#3B2A22", "#2E2A3A", "#24332F", "#3A2F1E", "#33262B", "#2A3238",
];

/// Stable tint per title so the same book always wears the same cover.
pub fn cover_tint(title: &str) -> &'static str {
    let hash = title.bytes().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as usize)
    });
    COVER_TINTS[hash % COVER_TINTS.len()]
}

#[component]
pub fn Cover(
    #[prop(into)] title: String,
    #[prop(into)] cover_url: String,
    #[prop(optional)] badge: Option<String>,
) -> impl IntoView {
    let tint = cover_tint(&title);
    // The typographic plate always renders beneath the artwork. A CSS
    // background that fails to load paints nothing, so the title shows
    // through without a broken-image glyph and without an error handler.
    let art = api::asset_url(&cover_url).map(|src| {
        format!(
            "background-image: url(\"{}\")",
            src.replace('"', "%22").replace(')', "%29")
        )
    });
    view! {
        <div class="cover" style=format!("--cover-tint: {tint}")>
            <span>{title}</span>
            {art.map(|style| view! { <span class="cover-art" style=style aria-hidden="true"></span> })}
            {badge.map(|label| view! {
                <span class="cover-badge" role="img" aria-label=label>{icons::download()}</span>
            })}
        </div>
    }
}

#[component]
pub fn BookCard(book: BookSummaryDto) -> impl IntoView {
    let (lang, _) = use_lang();
    let href = format!("/book/{}", book.id);
    let minutes = book.estimated_reading_minutes;
    let language = book.language.to_uppercase();
    let has_meta = minutes > 0;
    view! {
        <a href=href class="book-card">
            <Cover title=book.title.clone() cover_url=book.cover_url.clone()/>
            <div>
                <h3 class="book-title">{book.title.clone()}</h3>
                <p class="book-author">
                    {book.author.clone()}
                    {has_meta.then(|| view! { <span>{move || format!(" · {language} · {}", lang.get().duration(minutes))}</span> })}
                </p>
            </div>
        </a>
    }
}

#[component]
pub fn SkeletonCard() -> impl IntoView {
    view! {
        <div class="book-card" aria-hidden="true">
            <div class="skeleton skeleton-card"></div>
            <div class="skeleton skeleton-line" style="width: 80%"></div>
            <div class="skeleton skeleton-line" style="width: 55%"></div>
        </div>
    }
}
