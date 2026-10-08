//! Bottom tab bar for phones; hidden from 768px up by the stylesheet and
//! absent on book pages, where the action dock takes its place.

use crate::components::icons;
use crate::i18n::use_lang;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_query_map};

pub const ACCOUNT_ID: &str = "akun";

fn current(active: bool) -> Option<&'static str> {
    active.then_some("page")
}

#[component]
pub fn TabBar() -> impl IntoView {
    let (lang, _) = use_lang();
    let location = use_location();
    let query = use_query_map();
    let on_book_page = move || location.pathname.get().starts_with("/book/");

    let at_home = move || location.pathname.get() == "/";
    let searching = move || at_home() && query.read().get("search").is_some();
    let at_shelf = move || location.pathname.get() == "/me";
    let on_account =
        move || at_shelf() && location.hash.get().trim_start_matches('#') == ACCOUNT_ID;

    view! {
        <nav class="tabbar" aria-label=move || lang.get().text("nav_tabs") class:docked=on_book_page>
            <a href="/" class="tab" aria-current=move || current(at_home() && !searching())>
                {icons::home()}
                <span>{move || lang.get().text("nav_home")}</span>
            </a>
            <a href="/?search=1" class="tab" aria-current=move || current(searching())>
                {icons::search()}
                <span>{move || lang.get().text("nav_search")}</span>
            </a>
            <a href="/me" class="tab" aria-current=move || current(at_shelf() && !on_account())>
                {icons::shelf()}
                <span>{move || lang.get().text("nav_shelf")}</span>
            </a>
            <a href="/me#akun" class="tab" aria-current=move || current(on_account())>
                {icons::user()}
                <span>{move || lang.get().text("nav_profile")}</span>
            </a>
        </nav>
    }
}
