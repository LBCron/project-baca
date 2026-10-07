//! Curator page: EPUB upload and a live view of the ingestion job.

use crate::api;
use crate::components::icons;
use crate::components::session::use_session;
use crate::components::toast::use_toasts;
use crate::i18n::use_lang;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::JobStatusDto;
use std::time::Duration;

const JOB_PHASES: [(&str, &str); 6] = [
    ("queued", "job_queued"),
    ("parsing", "job_parsing"),
    ("chunking", "job_chunking"),
    ("embedding", "job_embedding"),
    ("summarizing", "job_summarizing"),
    ("published", "job_published"),
];

const POLL_INTERVAL_MS: u64 = 2000;

async fn upload_epub(file: web_sys::File, title: String, author: String) -> Result<String, String> {
    let form = web_sys::FormData::new().map_err(|e| format!("{e:?}"))?;
    form.append_with_blob_and_filename("file", &file, &file.name())
        .map_err(|e| format!("{e:?}"))?;
    form.append_with_str("title", &title)
        .map_err(|e| format!("{e:?}"))?;
    form.append_with_str("author", &author)
        .map_err(|e| format!("{e:?}"))?;
    let resp = gloo_net::http::Request::post(&format!("{}/admin/books/upload", api::api_base()))
        .header(
            "Authorization",
            &format!("Bearer {}", api::token().unwrap_or_default()),
        )
        .body(wasm_bindgen::JsValue::from(form))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let envelope = resp
        .json::<shared::ApiResponse<shared::UploadBookResponseDto>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope.data.map(|d| d.job_id).ok_or_else(|| {
        envelope
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "upload rejected".to_string())
    })
}

async fn job_status(job_id: &str) -> Result<JobStatusDto, String> {
    api::admin_job(job_id).await
}

fn phase_index(status: &str) -> usize {
    JOB_PHASES
        .iter()
        .position(|(key, _)| *key == status)
        .unwrap_or(0)
}

#[component]
fn JobCard(job: JobStatusDto) -> impl IntoView {
    let (lang, _) = use_lang();
    let failed = job.status == "failed";
    let reached = phase_index(&job.status);
    let short_id = job.job_id.chars().take(8).collect::<String>();
    view! {
        <article class="card" style="display: flex; flex-direction: column; gap: 16px" aria-live="polite">
            <div class="section-head" style="margin: 0">
                <h2 class="section-title" style="font-size: 1.2rem">{move || format!("{} {short_id}", lang.get().text("admin_job"))}</h2>
                <span class="tag">{format!("{}%", job.progress)}</span>
            </div>
            <div class="progress-bar"><span style=format!("width: {}%", job.progress.clamp(0, 100))></span></div>
            <ol class="job-steps" style="list-style: none; margin: 0; padding: 0">
                {JOB_PHASES.iter().enumerate().map(|(i, (_, label))| {
                    let done = i < reached || (i == reached && job.status == "published");
                    let active = i == reached && !done && !failed;
                    view! {
                        <li class="job-step" class:done=done class:active=active>
                            <span class="mark" aria-hidden="true">{done.then(icons::check)}</span>
                            <span>{move || lang.get().text(label)}</span>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ol>
            {failed.then(|| view! { <p class="form-error" role="alert">{move || lang.get().text("job_failed")}</p> })}
        </article>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    let toasts = use_toasts();
    let (title, set_title) = signal(String::new());
    let (author, set_author) = signal(String::new());
    let (file_name, set_file_name) = signal(None::<String>);
    let (job, set_job) = signal(None::<JobStatusDto>);
    let (busy, set_busy) = signal(false);
    let file_ref = NodeRef::<leptos::html::Input>::new();

    let poll = move |job_id: String| {
        let handle = set_interval_with_handle(
            move || {
                let id = job_id.clone();
                spawn_local(async move {
                    if let Ok(status) = job_status(&id).await {
                        set_job.set(Some(status));
                    }
                });
            },
            Duration::from_millis(POLL_INTERVAL_MS),
        );
        if let Ok(handle) = handle {
            Effect::new(move || {
                if let Some(j) = job.get() {
                    if j.status == "published" || j.status == "failed" {
                        handle.clear();
                    }
                }
            });
        }
    };

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let file = file_ref
            .get()
            .and_then(|i| i.files())
            .and_then(|f| f.get(0));
        let Some(file) = file else {
            toasts.error(lang.get_untracked().text("admin_choose_first"));
            return;
        };
        set_busy.set(true);
        let (t, a) = (title.get_untracked(), author.get_untracked());
        spawn_local(async move {
            match upload_epub(file, t, a).await {
                Ok(job_id) => {
                    match job_status(&job_id).await {
                        Ok(status) => set_job.set(Some(status)),
                        Err(e) => toasts.error(e),
                    }
                    poll(job_id);
                }
                Err(e) => toasts.error(e),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="container" style="padding-top: 20px">
            {move || match session.profile.get() {
                None if session.authed.get() => view! { <div class="skeleton" style="height: 200px"></div> }.into_any(),
                Some(p) if p.role == "admin" => view! {
                    <div class="admin-grid">
                        <form class="card" style="display: flex; flex-direction: column; gap: 16px" on:submit=submit>
                            <div>
                                <div class="kicker">{move || lang.get().text("admin_title")}</div>
                                <h1 class="section-title">{move || lang.get().text("admin_upload")}</h1>
                            </div>
                            <label class="upload-zone" class:has-file=move || file_name.get().is_some()>
                                {icons::upload()}
                                <span>{move || file_name.get().unwrap_or_else(|| lang.get().text("admin_drop"))}</span>
                                <input type="file" accept=".epub,application/epub+zip" node_ref=file_ref
                                    on:change=move |_| set_file_name.set(file_ref.get().and_then(|i| i.files()).and_then(|f| f.get(0)).map(|f| f.name()))/>
                            </label>
                            <label class="field">
                                {move || lang.get().text("admin_title_field")}
                                <input class="input" type="text" maxlength="255" prop:value=move || title.get() on:input=move |ev| set_title.set(event_target_value(&ev))/>
                            </label>
                            <label class="field">
                                {move || lang.get().text("admin_author_field")}
                                <input class="input" type="text" maxlength="255" prop:value=move || author.get() on:input=move |ev| set_author.set(event_target_value(&ev))/>
                            </label>
                            <button type="submit" class="btn btn-primary" disabled=move || busy.get()>
                                {move || if busy.get() { lang.get().text("admin_uploading") } else { lang.get().text("admin_queue") }}
                            </button>
                        </form>
                        {move || job.get().map(|j| view! { <JobCard job=j/> })}
                    </div>
                }.into_any(),
                _ => view! {
                    <div class="empty-state">
                        <img src="/assets/admin-sorting-pigeonholes.webp" alt=""/>
                        <p>{move || lang.get().text("admin_forbidden")}</p>
                        <a href="/" class="btn btn-ghost">{move || lang.get().text("back_to_catalog")}</a>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
