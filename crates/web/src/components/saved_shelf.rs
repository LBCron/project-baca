//! Books saved on this device, for readers the server cannot list them for:
//! a guest, or anyone offline.

use crate::components::book_card::Cover;
use crate::i18n::use_lang;
use crate::storage;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::BookDetailDto;

/// The device's saved books as a shelf, or nothing when there are none.
#[component]
pub fn SavedOnDevice() -> impl IntoView {
    let (lang, _) = use_lang();
    let (books, set_books) = signal(Vec::<BookDetailDto>::new());
    spawn_local(async move {
        if let Ok(saved) = storage::get_all::<BookDetailDto>(storage::STORE_OFFLINE_BOOKS).await {
            set_books.set(saved);
        }
    });
    move || {
        let saved = books.get();
        (!saved.is_empty()).then(|| {
            view! {
                <section class="saved-on-device" aria-labelledby="saved-device-title">
                    <div class="section-head">
                        <h2 id="saved-device-title" class="section-title">{move || lang.get().text("saved_books_device")}</h2>
                    </div>
                    <div class="shelf-grid">
                        {saved.into_iter().map(|b| view! {
                            <a href=format!("/book/{}", b.id) class="shelf-item" aria-label=b.title.clone()>
                                <Cover title=b.title.clone() cover_url=b.cover_url.clone()/>
                            </a>
                        }).collect::<Vec<_>>()}
                    </div>
                </section>
            }
        })
    }
}
