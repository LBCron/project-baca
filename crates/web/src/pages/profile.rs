//! Shelf and profile: streak, saved books, badges, quotes, sessions.

use crate::api;
use crate::components::book_card::Cover;
use crate::components::icons;
use crate::components::progress::ProgressBar;
use crate::components::session::use_session;
use crate::components::toast::use_toasts;
use crate::i18n::{use_lang, Lang};
use crate::storage;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{BookDetailDto, ReadingStreakDto, SavedQuoteResponseDto, UserBadgeDto};

#[component]
fn GuestShelf() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let (local, set_local) = signal(Vec::<storage::LocalQuoteRecord>::new());
    spawn_local(async move {
        if let Ok(items) =
            storage::get_all::<storage::LocalQuoteRecord>(storage::STORE_LOCAL_QUOTES).await
        {
            set_local.set(items);
        }
    });
    let share = move |text: String| {
        spawn_local(async move {
            let body = format!("\u{201C}{text}\u{201D} \u{2014} Rotaria");
            if !api::share_text("Rotaria", &body).await && api::copy_text(&body).await {
                toasts.info(lang.get_untracked().text("quote_saved"));
            }
        });
    };
    view! {
        <div class="profile">
            <div class="empty-state guest-shelf">
                <img src="/assets/cozy-reader-armchair-owl.webp" alt="" width="640" height="640"/>
                <h1 class="section-title">{move || lang.get().text("shelf_title")}</h1>
                <p>{move || lang.get().text("guest_shelf")}</p>
                <div class="sheet-actions centered">
                    <button class="btn btn-primary" on:click=move |_| session.open_sheet()>{move || lang.get().text("sign_in")}</button>
                    <a href="/" class="btn btn-ghost">{move || lang.get().text("back_to_catalog")}</a>
                </div>
            </div>
            {move || (!local.get().is_empty()).then(|| view! {
                <section aria-labelledby="local-quotes-title">
                    <div class="section-head">
                        <h2 id="local-quotes-title" class="section-title">{move || lang.get().text("saved_quotes")}</h2>
                        <span class="form-hint">{move || lang.get().text("quote_guest_hint")}</span>
                    </div>
                    <ol class="quote-results">
                        {local.get().into_iter().map(|q| {
                            let text = q.quote_text.clone();
                            let href = format!("/book/{}", q.book_id);
                            view! {
                                <li class="quote-row">
                                    <div class="quote-meta"><span>{move || lang.get().date_short(q.saved_at)}</span><span aria-hidden="true">"\u{2756}"</span></div>
                                    <blockquote>{q.quote_text.clone()}</blockquote>
                                    <div class="quote-actions">
                                        <button class="btn btn-primary" on:click=move |_| share(text.clone())>{icons::share()}<span>{move || lang.get().text("quote_share")}</span></button>
                                        <a href=href class="btn btn-ghost">{move || lang.get().text("quote_jump")}</a>
                                    </div>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ol>
                </section>
            })}
        </div>
    }
}

#[component]
fn StreakCard(streak: ReadingStreakDto) -> impl IntoView {
    let (lang, _) = use_lang();
    let days = streak.current_streak_days;
    let lit = days.clamp(0, 7) as usize;
    let today_done = streak.today_seconds >= streak.daily_threshold_seconds;
    view! {
        <section class="streak-card" aria-label="Streak">
            <div class="flame" aria-hidden="true">{icons::flame()}</div>
            <div class="streak-body">
                <span class="count">{move || lang.get().text_with("streak_days", "n", &days.to_string())}</span>
                <small>{move || if today_done { format!("{} \u{00B7} {}", lang.get().text_with("xp", "n", &streak.total_xp.to_string()), lang.get().duration((streak.total_reading_seconds / 60) as i32)) } else { lang.get().text("streak_hint") }}</small>
                <div class="week-dots" aria-hidden="true">
                    {(0..7).map(|i| {
                        let on = i < lit;
                        view! { <span class:on=on></span> }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

#[component]
fn DeviceList() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let (devices, set_devices) = signal(Vec::<shared::SessionDto>::new());
    let (others_armed, set_others_armed) = signal(false);
    let reload = move || {
        spawn_local(async move {
            if let Ok(list) = api::my_sessions().await {
                set_devices.set(list);
            }
        });
    };
    reload();
    let revoke = move |id: String, current: bool| {
        spawn_local(async move {
            match api::revoke_session(&id).await {
                Ok(_) if current => {
                    api::clear_token();
                    session.refresh();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Ok(_) => {
                    toasts.info(lang.get_untracked().text("device_revoked"));
                    set_devices.update(|all| all.retain(|d| d.id != id));
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
        });
    };
    let revoke_others = move |_| {
        if !others_armed.get_untracked() {
            set_others_armed.set(true);
            return;
        }
        set_others_armed.set(false);
        spawn_local(async move {
            match api::revoke_all(true).await {
                Ok(_) => {
                    toasts.info(lang.get_untracked().text("others_revoked"));
                    reload();
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
        });
    };
    view! {
        <div class="settings-list">
            <div class="section-head compact">
                <h3 class="section-title small">{move || lang.get().text("devices")}</h3>
                {move || (devices.get().len() > 1).then(|| view! {
                    <button class="btn btn-ghost" on:click=revoke_others>
                        {move || if others_armed.get() { lang.get().text("confirm_again") } else { lang.get().text("revoke_others_now") }}
                    </button>
                })}
            </div>
            <ul class="device-list">
                {move || devices.get().into_iter().map(|d| {
                    let id = d.id.clone();
                    let current = d.current;
                    let label = if d.device.is_empty() { lang.get().text("unknown_device") } else { d.device.clone() };
                    view! {
                        <li class="device-row" class:current=current>
                            <div class="device-main">
                                <span class="device-name">
                                    {label}
                                    {current.then(|| view! { <span class="tag">{move || lang.get().text("this_device")}</span> })}
                                </span>
                                <span class="manuscript-meta">
                                    {move || format!(
                                        "{}{}",
                                        if d.ip_prefix.is_empty() { String::new() } else { format!("{} \u{00B7} ", d.ip_prefix) },
                                        lang.get().text_with("last_seen", "t", &lang.get().date_short(d.last_seen_at))
                                    )}
                                </span>
                            </div>
                            <button class="btn btn-ghost" on:click=move |_| revoke(id.clone(), current)>
                                {move || lang.get().text("sign_out_device")}
                            </button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

#[component]
fn SessionsPanel() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let (current_pw, set_current_pw) = signal(String::new());
    let (new_pw, set_new_pw) = signal(String::new());
    let (revoke_others, set_revoke_others) = signal(true);
    let (busy, set_busy) = signal(false);
    let (revoke_armed, set_revoke_armed) = signal(false);

    let change_password = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_busy.set(true);
        spawn_local(async move {
            let req = shared::ChangePasswordRequest {
                current_password: current_pw.get_untracked(),
                new_password: new_pw.get_untracked(),
                revoke_other_sessions: Some(revoke_others.get_untracked()),
            };
            match api::change_password(&req).await {
                Ok(_) => {
                    toasts.info(lang.get_untracked().text("password_changed"));
                    set_current_pw.set(String::new());
                    set_new_pw.set(String::new());
                }
                Err(e) => toasts.error(
                    e.split_once(": ")
                        .map(|(_, m)| m.to_string())
                        .unwrap_or_else(|| lang.get_untracked().text("error_generic")),
                ),
            }
            set_busy.set(false);
        });
    };

    let revoke_all = move |_| {
        if !revoke_armed.get_untracked() {
            set_revoke_armed.set(true);
            return;
        }
        set_revoke_armed.set(false);
        spawn_local(async move {
            match api::revoke_all(false).await {
                Ok(_) => {
                    api::clear_token();
                    session.refresh();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
        });
    };

    view! {
        <section id="akun" class="card settings-list">
            <h2 class="section-title">{move || lang.get().text("sessions")}</h2>
            <DeviceList/>
            <form class="settings-list" on:submit=change_password>
                <label class="field">
                    <span>{move || lang.get().text("current_password")}</span>
                    <input class="input" type="password" autocomplete="current-password" required prop:value=move || current_pw.get() on:input=move |ev| set_current_pw.set(event_target_value(&ev))/>
                </label>
                <label class="field">
                    <span>{move || lang.get().text("new_password")}</span>
                    <input class="input" type="password" autocomplete="new-password" required minlength="8" maxlength="128" prop:value=move || new_pw.get() on:input=move |ev| set_new_pw.set(event_target_value(&ev))/>
                    <span class="form-hint">{move || lang.get().text("auth_password_hint")}</span>
                </label>
                <label class="field check">
                    <input type="checkbox" prop:checked=move || revoke_others.get() on:change=move |ev| set_revoke_others.set(event_target_checked(&ev))/>
                    <span>{move || lang.get().text("revoke_others")}</span>
                </label>
                <div class="sheet-actions">
                    <button type="submit" class="btn btn-primary" disabled=move || busy.get()>{move || lang.get().text("change_password")}</button>
                </div>
            </form>
            <div class="sheet-actions">
                <button class="btn btn-ghost" on:click=revoke_all>
                    {move || if revoke_armed.get() { lang.get().text("confirm_again") } else { lang.get().text("revoke_all") }}
                </button>
                <button class="btn btn-ghost" on:click=move |_| session.sign_out()>{move || lang.get().text("sign_out")}</button>
            </div>
        </section>
    }
}

fn badge_title(badge: &UserBadgeDto, lang: Lang) -> String {
    match lang {
        Lang::Id => badge.badge.title_id.clone(),
        Lang::En => badge.badge.title_en.clone(),
    }
}

fn badge_description(badge: &UserBadgeDto, lang: Lang) -> String {
    match lang {
        Lang::Id => badge.badge.description_id.clone(),
        Lang::En => badge.badge.description_en.clone(),
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let (streak, set_streak) = signal(None::<ReadingStreakDto>);
    let (badges, set_badges) = signal(Vec::<UserBadgeDto>::new());
    let (quotes, set_quotes) = signal(Vec::<SavedQuoteResponseDto>::new());
    let (offline_books, set_offline_books) = signal(Vec::<BookDetailDto>::new());
    let (active, set_active) = signal(None::<(uuid::Uuid, f32)>);

    Effect::new(move || {
        if !session.authed.get() {
            return;
        }
        spawn_local(async move {
            if let Ok(s) = api::my_streak().await {
                set_streak.set(Some(s));
            }
            if let Ok(list) = api::my_badges().await {
                set_badges.set(list);
            }
            if let Ok(saved) = api::my_quotes().await {
                set_quotes.set(saved);
            }
            if let Ok(Some(progress)) = api::active_progress().await {
                set_active.set(Some((progress.book_id, progress.completion_percentage)));
            }
        });
    });

    spawn_local(async move {
        if let Ok(books) = storage::get_all::<BookDetailDto>(storage::STORE_OFFLINE_BOOKS).await {
            set_offline_books.set(books);
        }
    });

    let share = move |text: String| {
        spawn_local(async move {
            let body = format!("\u{201C}{text}\u{201D} \u{2014} Rotaria");
            if !api::share_text("Rotaria", &body).await && api::copy_text(&body).await {
                toasts.info(lang.get_untracked().text("quote_saved"));
            }
        });
    };

    let subtitle = move || {
        let since = session
            .profile
            .get()
            .map(|p| {
                lang.get()
                    .text_with("reader_since", "date", &lang.get().month_year(p.created_at))
            })
            .unwrap_or_default();
        match streak.get() {
            Some(s) => format!(
                "{since} \u{00B7} {}",
                lang.get().text_with("xp", "n", &s.total_xp.to_string())
            ),
            None => since,
        }
    };

    view! {
        <div class="container">
            {move || if !session.authed.get() {
                view! { <GuestShelf/> }.into_any()
            } else {
                view! {
                    <div class="profile">
                        <header class="profile-head">
                            <div class="profile-identity">
                                <div class="avatar" aria-hidden="true">{move || session.initial()}</div>
                                <div class="profile-name">
                                    <h1 class="display">{move || session.profile.get().map(|p| p.display_name).unwrap_or_default()}</h1>
                                    <p class="form-hint">{subtitle}</p>
                                </div>
                            </div>
                            <a href="#akun" class="btn-icon" aria-label=move || lang.get().text("sessions")>{icons::settings()}</a>
                        </header>

                        {move || match streak.get() {
                            Some(s) => view! { <StreakCard streak=s/> }.into_any(),
                            None => view! { <div class="skeleton streak-skeleton"></div> }.into_any(),
                        }}

                        <section aria-labelledby="shelf-title">
                            <div class="section-head">
                                <h2 id="shelf-title" class="section-title">{move || lang.get().text("shelf_title")}</h2>
                                <span class="form-hint">{move || lang.get().text_with("saved_count", "n", &offline_books.get().len().to_string())}</span>
                            </div>
                            {move || if offline_books.get().is_empty() {
                                view! { <p class="form-hint">{move || lang.get().text("no_offline")}</p> }.into_any()
                            } else {
                                view! {
                                    <div class="shelf-grid">
                                        {offline_books.get().into_iter().map(|b| {
                                            let href = format!("/book/{}", b.id);
                                            let book_id = b.id;
                                            let percent = move || active.get().filter(|(id, _)| *id == book_id).map(|(_, p)| p);
                                            view! {
                                                <a href=href class="shelf-item" aria-label=b.title.clone()>
                                                    <Cover title=b.title.clone() cover_url=b.cover_url.clone() badge=lang.get_untracked().text("saved_offline")/>
                                                    {move || percent().map(|p| view! {
                                                        <ProgressBar percent=Signal::derive(move || p) label=Signal::derive(move || lang.get().text("reading_progress")) thin=true/>
                                                    })}
                                                </a>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }}
                        </section>

                        <section aria-labelledby="badges-title">
                            <div class="section-head"><h2 id="badges-title" class="section-title">{move || lang.get().text("badges")}</h2></div>
                            {move || if badges.get().is_empty() {
                                view! { <p class="form-hint">{move || lang.get().text("no_badges")}</p> }.into_any()
                            } else {
                                view! {
                                    <div class="badge-grid">
                                        {badges.get().iter().enumerate().map(|(i, b)| view! {
                                            <article class="badge-card">
                                                <span class="numeral">{crate::format::roman(i as i32 + 1)}</span>
                                                <h3>{badge_title(b, lang.get())}</h3>
                                                <p>{badge_description(b, lang.get())}</p>
                                                <span class="tag">{format!("+{} XP", b.badge.xp_reward)}</span>
                                            </article>
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }}
                        </section>

                        <section aria-labelledby="quotes-title">
                            <div class="section-head">
                                <h2 id="quotes-title" class="section-title">{move || lang.get().text("saved_quotes")}</h2>
                                {move || (!quotes.get().is_empty()).then(|| view! {
                                    <span class="form-hint">{move || lang.get().text_with("quotes_all", "n", &quotes.get().len().to_string())}</span>
                                })}
                            </div>
                            {move || if quotes.get().is_empty() {
                                view! { <p class="form-hint">{move || lang.get().text("no_quotes")}</p> }.into_any()
                            } else {
                                view! {
                                    <ol class="quote-results">
                                        {quotes.get().into_iter().map(|q| {
                                            let text = q.quote_text.clone();
                                            let href = format!("/book/{}", q.book_id);
                                            view! {
                                                <li class="quote-row">
                                                    <div class="quote-meta"><span>{move || lang.get().date_short(q.created_at)}</span><span aria-hidden="true">"\u{2756}"</span></div>
                                                    <blockquote>{q.quote_text.clone()}</blockquote>
                                                    <div class="quote-actions">
                                                        <button class="btn btn-primary" on:click=move |_| share(text.clone())>{icons::share()}<span>{move || lang.get().text("quote_share")}</span></button>
                                                        <a href=href class="btn btn-ghost">{move || lang.get().text("quote_jump")}</a>
                                                    </div>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ol>
                                }.into_any()
                            }}
                        </section>

                        <SessionsPanel/>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
