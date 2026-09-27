//! Project Baca — Leptos 0.7 WASM Single Page Application
//! Theme: Vintage Literary (1900–1950)

pub mod api;
pub mod pages;
pub mod storage;

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use pages::book::BookPage;
use pages::home::HomePage;

#[component]
pub fn App() -> impl IntoView {
    let (lang, set_lang) = signal("EN".to_string());

    view! {
        <Router>
            <Routes fallback=|| view! { <p class="hero-desc">"Manuscript not found."</p> }>
                <ParentRoute path=path!("") view=move || view! { <HomePage lang=lang set_lang=set_lang/> }>
                    <Route path=path!("") view=move || view! { <HomePage lang=lang set_lang=set_lang/> }/>
                    <Route path=path!("book/:id") view=BookPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
