//! Chrome around the catalog, book, profile, and admin routes: header, tab
//! bar, auth sheet, and toasts. The reader renders without it.

use crate::components::auth::AuthSheet;
use crate::components::header::SiteHeader;
use crate::components::session::use_session;
use crate::components::tabbar::TabBar;
use crate::components::toast::{use_toasts, ToastStack};
use crate::i18n::use_lang;
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn ShellLayout() -> impl IntoView {
    let session = use_session();
    let toasts = use_toasts();
    let (lang, _) = use_lang();
    let authed = Callback::new(move |_| {
        session.refresh();
        toasts.info(lang.get_untracked().text("auth_welcome"));
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
