//! Sign-in sheet: login, registration, and OTP steps plus guest merge.

use crate::components::icons;
use crate::i18n::use_lang;
use crate::{api, storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{
    GuestMergeRequest, GuestProgressRecord, LoginRequest, SignupRequest, VerifyOtpRequest,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Login,
    Register,
    Otp,
}

#[component]
pub fn AuthSheet(show: RwSignal<bool>, on_authed: Callback<()>) -> impl IntoView {
    let (lang, _) = use_lang();
    let (mode, set_mode) = signal(Mode::Login);
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (otp, set_otp) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (busy, set_busy) = signal(false);

    let merge_guest = move || {
        spawn_local(async move {
            if let Ok(records) =
                storage::get_all::<GuestProgressRecord>(storage::STORE_GUEST_PROGRESS).await
            {
                if !records.is_empty()
                    && api::merge_guest_progress(&GuestMergeRequest { records })
                        .await
                        .is_ok()
                {
                    let _ = storage::clear(storage::STORE_GUEST_PROGRESS).await;
                }
                api::drain_pending().await;
            }
            on_authed.run(());
        });
    };

    let close = move || {
        show.set(false);
        set_error.set(None);
        set_busy.set(false);
    };

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_busy.set(true);
        set_error.set(None);
        let (m, e, p, n, o) = (
            mode.get_untracked(),
            email.get_untracked().trim().to_string(),
            password.get_untracked(),
            name.get_untracked().trim().to_string(),
            otp.get_untracked().trim().to_string(),
        );
        spawn_local(async move {
            let result = match m {
                Mode::Register => api::signup(&SignupRequest {
                    display_name: n,
                    email: e,
                    password: p,
                })
                .await
                .map(|_| None),
                Mode::Otp => api::verify_otp(&VerifyOtpRequest { email: e, otp: o })
                    .await
                    .map(|t| Some((t.access_token, t.refresh_token))),
                Mode::Login => api::login(&LoginRequest {
                    email: e,
                    password: p,
                })
                .await
                .map(|t| Some((t.access_token, t.refresh_token))),
            };
            match result {
                Ok(Some((access, refresh))) => {
                    api::set_tokens(&access, &refresh);
                    set_password.set(String::new());
                    set_otp.set(String::new());
                    close();
                    merge_guest();
                }
                Ok(None) => set_mode.set(Mode::Otp),
                Err(e) => {
                    let message = if m == Mode::Login && e.contains("UNAUTHORIZED") {
                        lang.get_untracked().text("auth_login_failed")
                    } else {
                        friendly_error(&e, lang.get_untracked())
                    };
                    set_error.set(Some(message));
                }
            }
            set_busy.set(false);
        });
    };

    let title_key = move || match mode.get() {
        Mode::Login => "auth_title_login",
        Mode::Register => "auth_title_register",
        Mode::Otp => "auth_title_otp",
    };

    view! {
        {move || show.get().then(|| view! {
            <div class="sheet-backdrop" on:click=move |_| close()>
                <div
                    class="sheet"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="auth-title"
                    on:click=|ev| ev.stop_propagation()
                >
                    <div class="sheet-grip" aria-hidden="true"></div>
                    <div class="sheet-head">
                        <h2 id="auth-title" class="section-title">{move || lang.get().text(title_key())}</h2>
                        <button class="btn-icon" aria-label=move || lang.get().text("close") on:click=move |_| close()>{icons::close()}</button>
                    </div>
                    {move || (mode.get() != Mode::Otp).then(|| view! {
                        <div class="segmented" role="tablist">
                            <button role="tab" aria-selected=move || mode.get() == Mode::Login class:active=move || mode.get() == Mode::Login on:click=move |_| set_mode.set(Mode::Login)>{move || lang.get().text("sign_in")}</button>
                            <button role="tab" aria-selected=move || mode.get() == Mode::Register class:active=move || mode.get() == Mode::Register on:click=move |_| set_mode.set(Mode::Register)>{move || lang.get().text("register")}</button>
                        </div>
                    })}
                    <form on:submit=submit style="display: flex; flex-direction: column; gap: 16px">
                        {move || (mode.get() == Mode::Register).then(|| view! {
                            <label class="field">
                                {move || lang.get().text("auth_name")}
                                <input class="input" type="text" autocomplete="nickname" required minlength="2" maxlength="100"
                                    prop:value=move || name.get()
                                    on:input=move |ev| set_name.set(event_target_value(&ev))/>
                            </label>
                        })}
                        {move || (mode.get() != Mode::Otp).then(|| view! {
                            <label class="field">
                                {move || lang.get().text("auth_email")}
                                <input class="input" type="email" autocomplete="email" required inputmode="email"
                                    prop:value=move || email.get()
                                    on:input=move |ev| set_email.set(event_target_value(&ev))/>
                            </label>
                            <label class="field">
                                {move || lang.get().text("auth_password")}
                                <input class="input" type="password" required minlength="8" maxlength="128"
                                    autocomplete=move || if mode.get() == Mode::Register { "new-password" } else { "current-password" }
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))/>
                                {move || (mode.get() == Mode::Register).then(|| view! { <span class="form-hint">{move || lang.get().text("auth_password_hint")}</span> })}
                            </label>
                        })}
                        {move || (mode.get() == Mode::Otp).then(|| view! {
                            <p class="form-hint">{move || lang.get().text_with("auth_otp_hint", "email", &email.get())}</p>
                            <label class="field">
                                {move || lang.get().text("auth_otp")}
                                <input class="input otp-input" type="text" inputmode="numeric" pattern="[0-9]{6}" maxlength="6" autocomplete="one-time-code" required
                                    prop:value=move || otp.get()
                                    on:input=move |ev| set_otp.set(event_target_value(&ev))/>
                            </label>
                        })}
                        {move || error.get().map(|e| view! { <p class="form-error" role="alert">{e}</p> })}
                        {move || (mode.get() != Mode::Otp && !api::is_authed()).then(|| view! {
                            <p class="form-hint">{move || lang.get().text("auth_guest_merge")}</p>
                        })}
                        <div class="sheet-actions">
                            <button type="submit" class="btn btn-primary btn-block" disabled=move || busy.get()>
                                {move || if busy.get() { lang.get().text("auth_working") } else { lang.get().text("auth_continue") }}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        })}
    }
}

/// Turns the API layer's "CODE: message" strings into reader-facing copy.
fn friendly_error(raw: &str, lang: crate::i18n::Lang) -> String {
    if raw.contains("RATE_LIMITED") || raw.contains("SERVICE_UNAVAILABLE") {
        return raw
            .split_once(": ")
            .map(|(_, m)| m.to_string())
            .unwrap_or_else(|| raw.to_string());
    }
    if raw.contains("VALIDATION_FAILED") || raw.contains("CONFLICT") {
        return raw
            .split_once(": ")
            .map(|(_, m)| m.to_string())
            .unwrap_or_else(|| raw.to_string());
    }
    lang.text("error_generic")
}
