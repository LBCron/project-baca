//! Site header: brand, desktop nav, language and theme toggles, account.

use crate::components::anchor::scroll_to_hash;
use crate::components::icons;
use crate::components::session::use_session;
use crate::i18n::{apply_lang, use_lang, Lang};
use crate::theme::ShellTheme;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

pub const HOW_IT_WORKS_ID: &str = "cara-kerja";

fn current(active: bool) -> Option<&'static str> {
    active.then_some("page")
}

#[component]
pub fn SiteHeader() -> impl IntoView {
    let (lang, set_lang) = use_lang();
    let session = use_session();
    let location = use_location();
    let theme =
        use_context::<RwSignal<ShellTheme>>().unwrap_or_else(|| RwSignal::new(ShellTheme::load()));

    let at_home = move || location.pathname.get() == "/";
    let at_how_it_works =
        move || at_home() && location.hash.get().trim_start_matches('#') == HOW_IT_WORKS_ID;
    let on_catalog =
        move || (at_home() && !at_how_it_works()) || location.pathname.get().starts_with("/book/");
    let on_shelf = move || location.pathname.get() == "/me";

    view! {
        <header class="site-header">
            <div class="container site-header-inner">
                <a href="/" class="brand-title" aria-label="Rotaria">
                    {icons::brand_mark()}
                    <span>"Rotaria"</span>
                </a>
                <nav class="nav-links" aria-label=move || lang.get().text("nav_main")>
                    <a href="/" class="nav-link" aria-current=move || current(on_catalog())>{move || lang.get().text("catalog")}</a>
                    <a href="/me" class="nav-link" aria-current=move || current(on_shelf())>{move || lang.get().text("nav_shelf_long")}</a>
                    <a
                        href="/#cara-kerja"
                        class="nav-link"
                        aria-current=move || current(at_how_it_works())
                        on:click=move |_| request_animation_frame(|| scroll_to_hash(HOW_IT_WORKS_ID))
                    >
                        <span>{move || lang.get().text("how_it_works")}</span>
                        {icons::arrow_down()}
                    </a>
                </nav>
                <div class="header-tools">
                    <div class="lang-toggle" role="group" aria-label=move || lang.get().text("language")>
                        <button
                            class="lang-opt"
                            class:active=move || lang.get() == Lang::Id
                            aria-pressed=move || lang.get() == Lang::Id
                            on:click=move |_| apply_lang(set_lang, Lang::Id)
                        >"ID"</button>
                        <button
                            class="lang-opt"
                            class:active=move || lang.get() == Lang::En
                            aria-pressed=move || lang.get() == Lang::En
                            on:click=move |_| apply_lang(set_lang, Lang::En)
                        >"EN"</button>
                    </div>
                    <button
                        class="btn-icon"
                        aria-label=move || lang.get().text("theme_toggle")
                        on:click=move |_| {
                            let next = theme.get_untracked().toggle();
                            next.persist();
                            next.apply();
                            theme.set(next);
                        }
                    >
                        {move || if theme.get() == ShellTheme::Paper { icons::moon().into_any() } else { icons::sun().into_any() }}
                    </button>
                    {move || if session.authed.get() {
                        view! {
                            <a href="/me" class="avatar small header-signin" aria-label=move || lang.get().text("nav_profile")>{move || session.initial()}</a>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn btn-light header-signin" on:click=move |_| session.open_sheet()>
                                {move || lang.get().text("sign_in")}
                            </button>
                        }.into_any()
                    }}
                </div>
            </div>
        </header>
    }
}
