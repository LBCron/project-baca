//! Sign-in sheet: login, registration, and OTP steps plus guest merge.

use crate::components::focus::{close_after_event, manage_sheet_focus};
use crate::components::icons;
use crate::i18n::use_lang;
use crate::{api, storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{
    GuestMergeRequest, GuestProgressRecord, LoginRequest, SaveQuoteRequest, SaveQuotesBatchRequest,
    SignupRequest, VerifyOtpRequest, SAVE_QUOTES_BATCH_MAX,
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
    let (registration, set_registration) = signal(None::<uuid::Uuid>);
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
            merge_local_quotes().await;
            on_authed.run(());
        });
    };

    let close = move || {
        close_after_event(show);
        set_error.set(None);
        set_busy.set(false);
    };
    let close_ref = NodeRef::<leptos::html::Button>::new();
    manage_sheet_focus(show.into(), close_ref, close);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_busy.set(true);
        set_error.set(None);
        let (m, e, p, n, o, r) = (
            mode.get_untracked(),
            email.get_untracked().trim().to_string(),
            password.get_untracked(),
            name.get_untracked().trim().to_string(),
            otp.get_untracked().trim().to_string(),
            registration.get_untracked().unwrap_or_default(),
        );
        spawn_local(async move {
            let result = match m {
                Mode::Register => {
                    let req = SignupRequest {
                        display_name: n,
                        email: e,
                        password: p,
                    };
                    match api::common_password_refusal("password", &req.password) {
                        Some(refusal) => Err(refusal),
                        None => api::signup(&req).await,
                    }
                    .map(|id| {
                        set_registration.set(Some(id));
                        None
                    })
                }
                Mode::Otp => api::verify_otp(&VerifyOtpRequest {
                    email: e,
                    registration_id: r,
                    otp: o,
                })
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
                    } else if e.starts_with(shared::error_code::CONFLICT) {
                        // Only an existing account conflicts during signup.
                        lang.get_untracked().text("auth_account_exists")
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
                        <button class="btn-icon" node_ref=close_ref aria-label=move || lang.get().text("close") on:click=move |_| close()>{icons::close()}</button>
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

/// Pushes quotes kept on this device into the account, fifty at a time, and
/// forgets every quote the server has answered for: saved, already there, or
/// pointing at a book that no longer exists.
async fn merge_local_quotes() {
    let Ok(records) =
        storage::get_all::<storage::LocalQuoteRecord>(storage::STORE_LOCAL_QUOTES).await
    else {
        return;
    };
    for chunk in records.chunks(SAVE_QUOTES_BATCH_MAX) {
        let request = SaveQuotesBatchRequest {
            items: chunk
                .iter()
                .map(|r| SaveQuoteRequest {
                    book_id: r.book_id,
                    chapter_id: r.chapter_id,
                    quote_text: r.quote_text.clone(),
                })
                .collect(),
        };
        let Ok(response) = api::save_quotes_batch(&request).await else {
            continue;
        };
        for answered in &response.outcomes {
            if let Some(record) = chunk.get(answered.index) {
                let _ = storage::delete(storage::STORE_LOCAL_QUOTES, &record.key).await;
            }
        }
    }
}

/// Turns the API layer's "CODE: message" or "CODE[field]: message" strings
/// into copy in the reader's language. The server's English message never
/// reaches the screen; only a wait is read from it.
pub(crate) fn friendly_error(raw: &str, lang: crate::i18n::Lang) -> String {
    use shared::error_code as code;
    let (head, message) = raw.split_once(": ").unwrap_or((raw, ""));
    let (error, refused) = head
        .split_once('[')
        .map_or((head, ""), |(c, f)| (c, f.trim_end_matches(']')));
    let (field, reason) = refused.split_once(':').unwrap_or((refused, ""));
    let wait = || {
        let seconds = message
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse::<u64>().ok())
            .next_back()
            .unwrap_or(60);
        lang.wait_text(seconds)
    };
    match error {
        code::QUOTA_EXCEEDED => lang.text("quotes_full"),
        code::RATE_LIMITED => lang.text_with("rate_limited", "t", &wait()),
        code::AI_RATE_LIMITED => lang.text_with("err_ai_rate_limited", "t", &wait()),
        code::SERVICE_UNAVAILABLE => lang.text_with("err_service_unavailable", "t", &wait()),
        code::VALIDATION_FAILED => {
            let by_reason = format!("err_{reason}");
            let by_field = format!("err_field_{field}");
            if !reason.is_empty() && lang.has(&by_reason) {
                lang.text(&by_reason)
            } else if lang.has(&by_field) {
                lang.text(&by_field)
            } else {
                lang.text("err_validation_failed")
            }
        }
        known if code::ALL.contains(&known) => lang.text(&error_key(known)),
        _ => lang.text("error_generic"),
    }
}

/// The dictionary key holding the copy for an API error code.
pub(crate) fn error_key(code: &str) -> String {
    format!("err_{}", code.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::friendly_error;
    use crate::i18n::Lang;

    const SERVER_ENGLISH: &str = "Uploaded file is not a ZIP-based EPUB (missing PK signature)";

    #[test]
    fn every_api_error_code_has_copy_in_both_languages() {
        for code in shared::error_code::ALL {
            for lang in [Lang::Id, Lang::En] {
                let text = friendly_error(&format!("{code}: {SERVER_ENGLISH}"), lang);
                assert!(
                    !text.starts_with("err_"),
                    "{code} lacks a {lang:?} entry: {text}"
                );
                assert!(
                    !text.contains(SERVER_ENGLISH),
                    "{code} shows the server's text"
                );
            }
        }
    }

    #[test]
    fn a_refused_field_names_what_to_fix() {
        let raw = "VALIDATION_FAILED[new_password]: New password must be at least 8 characters";
        assert_eq!(
            friendly_error(raw, Lang::Id),
            Lang::Id.text("err_field_new_password")
        );
        let unknown = "VALIDATION_FAILED[records]: too many";
        assert_eq!(
            friendly_error(unknown, Lang::En),
            Lang::En.text("err_validation_failed")
        );
    }

    #[test]
    fn a_common_password_is_named_whatever_field_refused_it() {
        for field in ["password", "new_password"] {
            let raw = format!("VALIDATION_FAILED[{field}:common_password]: Password is too common");
            assert_eq!(
                friendly_error(&raw, Lang::Id),
                Lang::Id.text("err_common_password")
            );
        }
        let length = "VALIDATION_FAILED[password:length]: Password must be at least 8 characters";
        assert_eq!(
            friendly_error(length, Lang::En),
            Lang::En.text("err_field_password")
        );
    }

    #[test]
    fn a_form_refuses_a_common_password_before_sending() {
        let refusal =
            crate::api::common_password_refusal("password", "QWERTY123").expect("refused locally");
        assert_eq!(
            friendly_error(&refusal, Lang::Id),
            Lang::Id.text("err_common_password")
        );
    }

    #[test]
    fn a_wait_reads_as_a_duration() {
        let raw = "RATE_LIMITED: Rate limit exceeded: at most 5 attempts per 15 minutes. Try again in 857 seconds";
        assert_eq!(
            friendly_error(raw, Lang::Id),
            "Terlalu banyak permintaan dalam waktu singkat. Coba lagi dalam 15 menit."
        );
        // A daily budget waits hours, not 1440 minutes.
        let budget = "AI_RATE_LIMITED: daily budget spent. Try again in 86400 seconds";
        assert!(friendly_error(budget, Lang::Id).ends_with("dalam 24 jam."));
        assert_eq!(
            friendly_error("SOMETHING_NEW: x", Lang::En),
            Lang::En.text("error_generic")
        );
    }

    #[test]
    fn the_indonesian_copy_says_kamu() {
        for code in shared::error_code::ALL {
            let text = friendly_error(&format!("{code}: x"), Lang::Id).to_lowercase();
            assert!(!text.contains("anda"), "{code}: {text}");
        }
    }
}
