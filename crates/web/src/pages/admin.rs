//! Admin: EPUB upload (multipart) + ingestion job monitor (P7).

use crate::api;
use crate::components::header::SiteHeader;
use leptos::prelude::*;
use leptos::task::spawn_local;

async fn upload_epub(file: web_sys::File, title: String, author: String) -> Result<String, String> {
    let form = web_sys::FormData::new().map_err(|e| format!("{e:?}"))?;
    form.append_with_blob_and_filename("file", &file, &file.name())
        .map_err(|e| format!("{e:?}"))?;
    form.append_with_str("title", &title)
        .map_err(|e| format!("{e:?}"))?;
    form.append_with_str("author", &author)
        .map_err(|e| format!("{e:?}"))?;
    let body = wasm_bindgen::JsValue::from(form);
    let resp =
        gloo_net::http::Request::post(&format!("{}/admin/books/upload", crate::api::api_base()))
            .header(
                "Authorization",
                &format!("Bearer {}", api::token().unwrap_or_default()),
            )
            .body(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
    let envelope = resp
        .json::<shared::ApiResponse<shared::UploadBookResponseDto>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope
        .data
        .map(|d| d.job_id)
        .ok_or_else(|| "upload rejected".to_string())
}

async fn job_status(job_id: &str) -> Result<shared::JobStatusDto, String> {
    let url = format!("{}/admin/jobs/{job_id}", crate::api::api_base());
    let builder = gloo_net::http::Request::get(&url);
    let authed = match api::token() {
        Some(t) => builder.header("Authorization", &format!("Bearer {t}")),
        None => builder,
    };
    let envelope = authed
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<shared::ApiResponse<shared::JobStatusDto>>()
        .await
        .map_err(|e| e.to_string())?;
    envelope.data.ok_or_else(|| "job not found".to_string())
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (author, set_author) = signal(String::new());
    let (job, set_job) = signal(None::<shared::JobStatusDto>);
    let (error, set_error) = signal(None::<String>);
    let (busy, set_busy) = signal(false);
    let file_ref = NodeRef::<leptos::html::Input>::new();

    let submit = move |_| {
        let files = file_ref.get().and_then(|i| i.files());
        let file = files.and_then(|f| f.get(0));
        let Some(file) = file else {
            set_error.set(Some("Choose an .epub file first.".to_string()));
            return;
        };
        set_busy.set(true);
        set_error.set(None);
        let (t, a) = (title.get_untracked(), author.get_untracked());
        spawn_local(async move {
            match upload_epub(file, t, a).await {
                Ok(job_id) => match job_status(&job_id).await {
                    Ok(status) => set_job.set(Some(status)),
                    Err(e) => set_error.set(Some(e)),
                },
                Err(e) => set_error.set(Some(e)),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="app-container">
            <SiteHeader/>
            <div class="catalog-section-title"><span>"II — Admin Upload"</span></div>
            {move || error.get().map(|e| view! { <p class="form-error">{e}</p> })}
            <label>"EPUB file (max 50 MB)"
                <input type="file" accept=".epub" node_ref=file_ref/>
            </label>
            <label>"Title"
                <input type="text" prop:value=move || title.get()
                    on:input=move |ev| set_title.set(event_target_value(&ev))/>
            </label>
            <label>"Author"
                <input type="text" prop:value=move || author.get()
                    on:input=move |ev| set_author.set(event_target_value(&ev))/>
            </label>
            <div class="book-actions">
                <button class="btn-read" on:click=submit disabled=move || busy.get()>
                    {move || if busy.get() { "Uploading…" } else { "Queue Ingestion" }}
                </button>
            </div>
            {move || job.get().map(|j| view! {
                <section class="book-card">
                    <div class="book-tag">{format!("{} — {}%", j.status, j.progress)}</div>
                    <h2 class="book-title">{format!("Job {}", &j.job_id[..8.min(j.job_id.len())])}</h2>
                    <p class="book-synopsis">{format!("Book {}", j.book_id)}</p>
                </section>
            })}
        </div>
    }
}
