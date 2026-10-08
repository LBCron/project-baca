//! Chrome around the catalog, book, profile, and admin routes: header, tab
//! bar, auth sheet, and toasts. The reader renders without it.

use crate::components::anchor::scroll_to_hash;
use crate::components::auth::AuthSheet;
use crate::components::header::SiteHeader;
use crate::components::session::use_session;
use crate::components::tabbar::TabBar;
use crate::components::toast::{use_toasts, ToastStack};
use crate::i18n::use_lang;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_location;

#[component]
pub fn ShellLayout() -> impl IntoView {
    let session = use_session();
    let toasts = use_toasts();
    let (lang, _) = use_lang();
    let location = use_location();
    let authed = Callback::new(move |_| {
        session.refresh();
        toasts.info(lang.get_untracked().text("auth_welcome"));
    });

    // The router's own hash scroll runs as the new view mounts; the target
    // may still be missing or laid out above its final place, so repeat it
    // on the next frame whenever the hash or the route changes.
    Effect::new(move || {
        let hash = location.hash.get();
        location.pathname.track();
        if hash.trim_start_matches('#').is_empty() {
            return;
        }
        request_animation_frame(move || scroll_to_hash(&hash));
    });

    view! {
        <div class="shell">
            <SiteHeader/>
            <main class="shell-main">
                <Outlet/>
            </main>
            <TabBar/>
            <AuthSheet show=session.sheet_open on_authed=authed/>
            <ToastStack toasts=toasts/>
        </div>
    }
}
