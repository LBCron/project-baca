//! Component story gallery (Task 11e): isolated render target for
//! Playwright component tests at `/__gallery?story=<name>`. Dev-only route;
//! production builds strip it via `#[cfg(debug_assertions)]` registration
//! (see `lib.rs`). Stories own all state; tests assert through the DOM.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::components::auth::AuthModal;
use crate::components::header::SiteHeader;
use crate::components::insights::{AtomicCards, CatchupRecap, QuoteFinder};

/// Renders the story named by `?story=`. Unknown names render a plain
/// marker so tests fail loudly instead of asserting on an empty page.
/// Release builds render nothing (gallery is a dev/test route).
#[component]
pub fn GalleryPage() -> impl IntoView {
    #[cfg(not(debug_assertions))]
    return view! { <p class="hero-desc">"Manuscript not found."</p> }.into_any();
    #[cfg(debug_assertions)]
    return gallery_body().into_any();
}

/// Debug-only gallery body (kept out of release builds).
#[cfg(debug_assertions)]
fn gallery_body() -> impl IntoView {
    let query = use_query_map();
    let story = move || query.read().get("story").unwrap_or_default();
    let show = RwSignal::new(true);
    let noop = Callback::new(|_| {});
    view! {
        <div id="gallery-root">
            {move || match story().as_str() {
                "site-header" => view! { <SiteHeader /> }.into_any(),
                "auth-modal" => view! { <AuthModal show=show on_authed=noop /> }.into_any(),
                "quote-finder" => view! {
                    <QuoteFinder book_id="00000000-0000-0000-0000-000000000000".to_string() show=show />
                }
                .into_any(),
                "atomic-cards" => view! {
                    <AtomicCards
                        book_id="00000000-0000-0000-0000-000000000000".to_string()
                        chapter=1
                        show=show
                    />
                }
                .into_any(),
                "recap" => view! {
                    <CatchupRecap
                        book_id="00000000-0000-0000-0000-000000000000".to_string()
                        chapter=2
                        show=show
                    />
                }
                .into_any(),
                other => {
                    view! { <p id="gallery-unknown">{format!("unknown story: {other}")}</p> }
                        .into_any()
                }
            }}
        </div>
    }
}
