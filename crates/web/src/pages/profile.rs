//! Profile: stats, streak, badges, saved quotes, logout (P4/P5/P6).

use crate::api;
use crate::components::header::SiteHeader;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{SavedQuoteResponseDto, UserBadgeDto, UserProfileDto};

#[component]
pub fn ProfilePage() -> impl IntoView {
    let (profile, set_profile) = signal(None::<UserProfileDto>);
    let (badges, set_badges) = signal(Vec::<UserBadgeDto>::new());
    let (quotes, set_quotes) = signal(Vec::<SavedQuoteResponseDto>::new());
    let (error, set_error) = signal(None::<String>);
    let (current_pw, set_current_pw) = signal(String::new());
    let (new_pw, set_new_pw) = signal(String::new());
    let (revoke_others, set_revoke_others) = signal(true);
    let (session_msg, set_session_msg) = signal(None::<String>);
    let (revoke_armed, set_revoke_armed) = signal(false);

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

    let change_password = move |_| {
        spawn_local(async move {
            let req = shared::ChangePasswordRequest {
                current_password: current_pw.get_untracked(),
                new_password: new_pw.get_untracked(),
                revoke_other_sessions: Some(revoke_others.get_untracked()),
            };
            match api::change_password(&req).await {
                Ok(_) => {
                    set_session_msg.set(Some("Password changed.".to_string()));
                    set_current_pw.set(String::new());
                    set_new_pw.set(String::new());
                }
                Err(e) => set_session_msg.set(Some(e)),
            }
        });
    };

    let revoke_all = move |_| {
        if !revoke_armed.get_untracked() {
            set_revoke_armed.set(true);
            return;
        }
        set_revoke_armed.set(false);
        spawn_local(async move {
            match api::revoke_all().await {
                Ok(_) => {
                    api::clear_token();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(e) => set_session_msg.set(Some(e)),
            }
        });
    };

    view! {
        <div class="app-container">
            <SiteHeader/>
            {move || match profile.get() {
                None => match error.get() {
                    None => view! { <p class="hero-desc">"Loading profile…"</p> }.into_any(),
                    Some(_) => view! {
                        <section class="book-overview">
                            <div class="catalog-section-title"><span>"Your Shelf"</span></div>
                            <p class="hero-desc">"Sign in to see your shelf, streaks, and saved quotes."</p>
                            <div class="book-actions">
                                <a href="/" class="btn-read"><span>"Back to Catalog"</span></a>
                            </div>
                        </section>
                    }.into_any(),
                },
                Some(me) => view! {
                    <section>
                        <div class="catalog-section-title"><span>{me.display_name.clone()}</span></div>
                        <p class="book-author">{me.email.clone()}</p>
                        <div class="book-actions">
                            <button class="btn-ghost" on:click=logout>"Log Out"</button>
                        </div>
                        <div class="catalog-section-title"><span>"❖ Sessions"</span></div>
                        <div class="session-panel">
                            <label>
                                "Current password"
                                <input type="password" class="search-bar" prop:value=move || current_pw.get()
                                    on:input=move |ev| set_current_pw.set(event_target_value(&ev))/>
                            </label>
                            <label>
                                "New password (min 8 chars)"
                                <input type="password" class="search-bar" prop:value=move || new_pw.get()
                                    on:input=move |ev| set_new_pw.set(event_target_value(&ev))/>
                            </label>
                            <label class="session-check">
                                <input type="checkbox" prop:checked=move || revoke_others.get()
                                    on:change=move |ev| set_revoke_others.set(event_target_checked(&ev))/>
                                "Sign out other sessions"
                            </label>
                            <div class="book-actions">
                                <button class="btn-read" on:click=change_password>"Change Password"</button>
                                <button class="btn-ghost" on:click=revoke_all>
                                    {move || if revoke_armed.get() { "Click again to confirm" } else { "Revoke All Sessions" }}
                                </button>
                            </div>
                            {move || session_msg.get().map(|m| view! {
                                <p class="hero-desc">{m}</p>
                            })}
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
