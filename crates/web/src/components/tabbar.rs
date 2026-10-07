//! Bottom tab bar for phones; hidden from 768px up by the stylesheet and
//! absent on book pages, where the action dock takes its place.

use crate::components::icons;
use crate::i18n::use_lang;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn TabBar() -> impl IntoView {
    let (lang, _) = use_lang();
    let location = use_location();
    let current = move |path: &str| (location.pathname.get() == path).then_some("page");
    let on_book_page = move || location.pathname.get().starts_with("/book/");

    view! {
        <nav class="tabbar" aria-label="Primary" class:docked=on_book_page>
            <a href="/" class="tab" aria-current=move || current("/")>
                {icons::home()}
                <span>{move || lang.get().text("nav_home")}</span>
            </a>
            <a href="/?search=1" class="tab">
                {icons::search()}
                <span>{move || lang.get().text("nav_search")}</span>
            </a>
            <a href="/me" class="tab" aria-current=move || current("/me")>
                {icons::shelf()}
                <span>{move || lang.get().text("nav_shelf")}</span>
            </a>
            <a href="/me#akun" class="tab">
                {icons::user()}
                <span>{move || lang.get().text("nav_profile")}</span>
            </a>
        </nav>
    }
}
