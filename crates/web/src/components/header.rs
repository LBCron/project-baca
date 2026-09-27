//! Shared site header: brand mark, nav, language toggle.

use crate::i18n::{apply_lang, lang_signal};
use leptos::prelude::*;

#[component]
pub fn SiteHeader() -> impl IntoView {
    let (lang, set_lang) = lang_signal();
    view! {
        <header class="header-vintage">
            <a href="/" class="brand-title">
                <img src="/assets/rotaria-windmill.svg" alt="Rotaria" class="brand-mark"/>
                <span>"Rotaria"</span>
            </a>
            <nav class="nav-links">
                <a href="/" class="nav-link">{move || lang.get().text("catalog")}</a>
                <a href="/me" class="nav-link">{move || lang.get().text("sign_in")}</a>
                <div class="lang-toggle" role="group" aria-label="Language">
                    <button
                        class="lang-opt"
                        class:active=move || lang.get() == crate::i18n::Lang::Id
                        on:click=move |_| apply_lang(set_lang, crate::i18n::Lang::Id)
                    >"ID"</button>
                    <button
                        class="lang-opt"
                        class:active=move || lang.get() == crate::i18n::Lang::En
                        on:click=move |_| apply_lang(set_lang, crate::i18n::Lang::En)
                    >"EN"</button>
                </div>
            </nav>
        </header>
    }
}
