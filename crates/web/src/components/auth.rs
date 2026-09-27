//! Signup, OTP, login, guest progress merge.

use crate::{api, storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{
    GuestMergeRequest, GuestProgressRecord, LoginRequest, SignupRequest, VerifyOtpRequest,
};

#[component]
pub fn AuthModal(show: RwSignal<bool>, on_authed: Callback<()>) -> impl IntoView {
    let (mode, set_mode) = signal("login".to_string());
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
                if !records.is_empty() {
                    let _ = api::merge_guest_progress(&GuestMergeRequest { records }).await;
                    let _ = storage::clear(storage::STORE_GUEST_PROGRESS).await;
                }
            }
            on_authed.run(());
        });
    };

    let submit = move |_| {
        set_busy.set(true);
        set_error.set(None);
        let (m, e, p, n, o) = (
            mode.get_untracked(),
            email.get_untracked(),
            password.get_untracked(),
            name.get_untracked(),
            otp.get_untracked(),
        );
        spawn_local(async move {
            let result = if m == "signup" {
                api::signup(&SignupRequest {
                    display_name: n,
                    email: e,
                    password: p,
                })
                .await
                .map(|_| None)
            } else if m == "otp" {
                api::verify_otp(&VerifyOtpRequest { email: e, otp: o })
                    .await
                    .map(|t| Some(t.access_token))
                    .map_err(|e| e)
            } else {
                api::login(&LoginRequest {
                    email: e,
                    password: p,
                })
                .await
                .map(|t| Some(t.access_token))
                .map_err(|e| e)
            };
            match result {
                Ok(token) => {
                    if let Some(t) = token {
                        api::set_token(&t);
                        show.set(false);
                        merge_guest();
                    } else {
                        set_mode.set("otp".to_string());
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_busy.set(false);
        });
    };

    view! {
        {move || show.get().then(|| view! {
            <div class="modal-backdrop">
                <div class="modal-vintage">
                    <div class="catalog-section-title"><span>"Sign In"</span></div>
                    {move || error.get().map(|e| view! { <p class="form-error">{e}</p> })}
                    <label>"Name (signup only)"
                        <input type="text" prop:value=move || name.get()
                            on:input=move |ev| set_name.set(event_target_value(&ev))/>
                    </label>
                    <label>"Email"
                        <input type="email" prop:value=move || email.get()
                            on:input=move |ev| set_email.set(event_target_value(&ev))/>
                    </label>
                    {move || (mode.get() != "otp").then(|| view! {
                        <label>"Password"
                            <input type="password" prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))/>
                        </label>
                    })}
                    {move || (mode.get() == "otp").then(|| view! {
                        <label>"6-digit OTP"
                            <input type="text" maxlength="6" prop:value=move || otp.get()
                                on:input=move |ev| set_otp.set(event_target_value(&ev))/>
                        </label>
                    })}
                    <div class="book-actions">
                        <button class="btn-read" on:click=submit disabled=move || busy.get()>
                            {move || if busy.get() { "Working…" } else { "Continue" }}
                        </button>
                        <button class="lang-switch" on:click=move |_| show.set(false)>"Cancel"</button>
                    </div>
                    <div class="auth-switch">
                        <button class="nav-link" on:click=move |_| set_mode.set("login".to_string())>"Login"</button>
                        <span>" ❖ "</span>
                        <button class="nav-link" on:click=move |_| set_mode.set("signup".to_string())>"Register"</button>
                    </div>
                </div>
            </div>
        })}
    }
}
