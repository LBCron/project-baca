//! Rotaria: Leptos 0.7 WASM single-page reader.

pub mod api;
pub mod components;
pub mod format;
pub mod i18n;
pub mod pages;
pub mod storage;
pub mod theme;

#[cfg(test)]
mod unit_tests;

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use components::session::Session;
use components::shell::ShellLayout;
use components::toast::Toasts;
use pages::admin::AdminPage;
use pages::book::BookPage;
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::profile::ProfilePage;
use pages::reader::ReaderPage;
use theme::ShellTheme;

#[component]
fn NotFound() -> impl IntoView {
    let (lang, _) = i18n::use_lang();
    view! {
        <div class="container empty-state" style="padding-top: 64px">
            <p>{move || lang.get().text("not_found")}</p>
            <a href="/" class="btn btn-ghost">{move || lang.get().text("back_to_catalog")}</a>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    i18n::provide_lang();
    let theme = RwSignal::new(ShellTheme::load());
    theme.get_untracked().apply();
    provide_context(theme);
    Toasts::provide();
    Session::provide();

    view! {
        <Router>
            <Routes fallback=NotFound>
                <ParentRoute path=path!("") view=ShellLayout>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("book/:id") view=BookPage/>
                    <Route path=path!("me") view=ProfilePage/>
                    <Route path=path!("admin") view=AdminPage/>
                    <Route path=path!("__gallery") view=GalleryPage/>
                </ParentRoute>
                <Route path=path!("read/:id") view=ReaderPage/>
            </Routes>
        </Router>
    }
}
