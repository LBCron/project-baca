//! Project Baca — Leptos 0.7 WASM Single Page Application
//! Theme: Vintage Literary (1900–1950)

pub mod api;
pub mod components;
pub mod i18n;
pub mod pages;
pub mod storage;

use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::path;

use pages::admin::AdminPage;
use pages::book::BookPage;
use pages::home::HomePage;
use pages::profile::ProfilePage;
use pages::reader::ReaderPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <p class="hero-desc">"Manuscript not found."</p> }>
                <ParentRoute path=path!("") view=Outlet>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("book/:id") view=BookPage/>
                    <Route path=path!("read/:id") view=ReaderPage/>
                    <Route path=path!("me") view=ProfilePage/>
                    <Route path=path!("admin") view=AdminPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
