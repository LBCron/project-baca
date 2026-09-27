//! Profile: stats, streak, badges, saved quotes, logout (P4/P5/P6).

use crate::api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{SavedQuoteResponseDto, UserBadgeDto, UserProfileDto};

#[component]
pub fn ProfilePage() -> impl IntoView {
    let (profile, set_profile) = signal(None::<UserProfileDto>);
    let (badges, set_badges) = signal(Vec::<UserBadgeDto>::new());
    let (quotes, set_quotes) = signal(Vec::<SavedQuoteResponseDto>::new());
    let (error, set_error) = signal(None::<String>);

    spawn_local(async move {
        match api::me().await {
            Ok(me) => set_profile.set(Some(me)),
            Err(e) => set_error.set(Some(e)),
        }
        if let Ok(list) = api::my_badges().await {
            set_badges.set(list);
        }
        if let Ok(saved) = api::my_quotes().await {
            set_quotes.set(saved);
        }
    });

    let logout = move |_| {
        spawn_local(async move {
            let _ = api::logout().await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        });
    };

    view! {
        <div class="app-container">
            <header class="header-vintage">
                <a href="/" class="brand-title">
                    <img src="/assets/rotaria-windmill.svg" alt="Rotaria" class="brand-mark"/>
                    <span>"Rotaria"</span>
                </a>
            </header>
            {move || match profile.get() {
                None => match error.get() {
                    None => view! { <p class="hero-desc">"Loading profile…"</p> }.into_any(),
                    Some(_) => view! { <p class="hero-desc">"Sign in to see your shelf."</p> }.into_any(),
                },
                Some(me) => view! {
                    <section>
                        <div class="catalog-section-title"><span>{me.display_name.clone()}</span></div>
                        <p class="book-author">{me.email.clone()}</p>
                        <div class="book-actions">
                            <button class="lang-switch" on:click=logout>"Log Out"</button>
                        </div>
                        <div class="catalog-section-title"><span>"❖ Badges"</span></div>
                        <div class="book-grid">
                            {badges.get().into_iter().map(|b| view! {
                                <div class="book-card">
                                    <div class="book-tag">{b.badge.title_en.clone()}</div>
                                    <p class="book-synopsis">{b.badge.description_en.clone()}</p>
                                    <div class="book-card-footer"><span>{format!("+{} XP", b.badge.xp_reward)}</span></div>
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                        <div class="catalog-section-title"><span>"❖ Saved Quotes"</span></div>
                        <ol class="quote-results">
                            {quotes.get().into_iter().map(|q| view! {
                                <li class="quote-row"><p>{q.quote_text.clone()}</p></li>
                            }).collect::<Vec<_>>()}
                        </ol>
                    </section>
                }.into_any(),
            }}
        </div>
    }
}
