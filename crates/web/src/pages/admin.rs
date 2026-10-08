//! Curator desk: manuscripts in every status behind the lifecycle guard, the
//! ingestion queue with its dead letters, and the drop-off funnel.

use crate::api;
use crate::components::icons;
use crate::components::session::use_session;
use crate::components::toast::use_toasts;
use crate::format::roman;
use crate::i18n::use_lang;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{AdminBookRowDto, DlqEntryDto, DropOffPointDto, JobStatusDto};
use std::time::Duration;

const JOB_PHASES: [(&str, &str); 6] = [
    ("queued", "job_queued"),
    ("parsing", "job_parsing"),
    ("chunking", "job_chunking"),
    ("embedding", "job_embedding"),
    ("summarizing", "job_summarizing"),
    ("published", "job_published"),
];

const STATUSES: [(&str, &str); 4] = [
    ("draft", "status_draft"),
    ("processing", "status_processing"),
    ("published", "status_published"),
    ("archived", "status_archived"),
];

const POLL_INTERVAL_MS: u64 = 2000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Desk {
    Manuscripts,
    Queue,
    Retention,
}

fn status_key(status: &str) -> &'static str {
    STATUSES
        .iter()
        .find(|(s, _)| *s == status)
        .map(|(_, key)| *key)
        .unwrap_or("status_draft")
}

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
    envelope
        .data
        .map(|d| d.job_id)
        .ok_or_else(|| envelope.error.map(|e| e.message).unwrap_or_default())
}

fn upload_error(message: &str, lang: &crate::i18n::Lang) -> String {
    if message.is_empty() {
        lang.text("upload_rejected")
    } else {
        message.to_string()
    }
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
        <article class="card job-card" aria-live="polite">
            <div class="section-head compact">
                <h2 class="section-title small">{move || format!("{} {short_id}", lang.get().text("admin_job"))}</h2>
                <span class="tag">{format!("{}%", job.progress)}</span>
            </div>
            <div class="progress-bar"><span style=format!("width: {}%", job.progress.clamp(0, 100))></span></div>
            <ol class="job-steps">
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
fn UploadCard(job: RwSignal<Option<JobStatusDto>>) -> impl IntoView {
    let (lang, _) = use_lang();
    let toasts = use_toasts();
    let (title, set_title) = signal(String::new());
    let (author, set_author) = signal(String::new());
    let (file_name, set_file_name) = signal(None::<String>);
    let (busy, set_busy) = signal(false);
    let file_ref = NodeRef::<leptos::html::Input>::new();
    let poll_handle = StoredValue::new(None::<IntervalHandle>);
    on_cleanup(move || {
        if let Some(handle) = poll_handle.try_get_value().flatten() {
            handle.clear();
        }
    });

    let poll = move |job_id: String| {
        let handle = set_interval_with_handle(
            move || {
                let id = job_id.clone();
                spawn_local(async move {
                    if let Ok(status) = api::admin_job(&id).await {
                        job.set(Some(status));
                    }
                });
            },
            Duration::from_millis(POLL_INTERVAL_MS),
        );
        if let Ok(handle) = handle {
            poll_handle.set_value(Some(handle));
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
                    match api::admin_job(&job_id).await {
                        Ok(status) => job.set(Some(status)),
                        Err(e) => toasts.error(upload_error(&e, &lang.get_untracked())),
                    }
                    poll(job_id);
                }
                Err(e) => toasts.error(upload_error(&e, &lang.get_untracked())),
            }
            set_busy.set(false);
        });
    };

    view! {
        <form class="card upload-card" on:submit=submit>
            <h2 class="section-title small">{move || lang.get().text("admin_upload")}</h2>
            <label class="upload-zone" class:has-file=move || file_name.get().is_some()>
                {icons::upload()}
                <span>{move || file_name.get().unwrap_or_else(|| lang.get().text("admin_drop"))}</span>
                <input type="file" accept=".epub,application/epub+zip" node_ref=file_ref
                    on:change=move |_| set_file_name.set(file_ref.get().and_then(|i| i.files()).and_then(|f| f.get(0)).map(|f| f.name()))/>
            </label>
            <label class="field">
                <span>{move || lang.get().text("admin_title_field")}</span>
                <input class="input" type="text" maxlength="255" prop:value=move || title.get() on:input=move |ev| set_title.set(event_target_value(&ev))/>
            </label>
            <label class="field">
                <span>{move || lang.get().text("admin_author_field")}</span>
                <input class="input" type="text" maxlength="255" prop:value=move || author.get() on:input=move |ev| set_author.set(event_target_value(&ev))/>
            </label>
            <button type="submit" class="btn btn-primary" disabled=move || busy.get()>
                {move || if busy.get() { lang.get().text("admin_uploading") } else { lang.get().text("admin_queue") }}
            </button>
        </form>
    }
}

#[component]
fn ManuscriptRow(row: AdminBookRowDto, on_updated: Callback<AdminBookRowDto>) -> impl IntoView {
    let (lang, _) = use_lang();
    let toasts = use_toasts();
    let (armed, set_armed) = signal(false);
    let (busy, set_busy) = signal(false);
    let id = row.id;
    let status = row.status.clone();
    let status_for_tag = row.status.clone();
    let archivable = status != "archived";
    let published = status == "published";
    let (chapters, chunks) = (row.chapter_count, row.chunk_count);
    let archive = move |_| {
        if !armed.get_untracked() {
            set_armed.set(true);
            return;
        }
        set_busy.set(true);
        spawn_local(async move {
            match api::admin_set_status(id, "archived").await {
                Ok(updated) => {
                    toasts.info(lang.get_untracked().text("archived_done"));
                    on_updated.run(updated);
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
            set_busy.set(false);
            set_armed.set(false);
        });
    };
    view! {
        <li class="manuscript-row">
            <div class="manuscript-main">
                {if published {
                    view! { <a href=format!("/book/{id}") class="manuscript-title">{row.title.clone()}</a> }.into_any()
                } else {
                    view! { <span class="manuscript-title">{row.title.clone()}</span> }.into_any()
                }}
                <span class="manuscript-meta">{format!("{} \u{00B7} {}", row.author, row.language.to_uppercase())}</span>
                <span class="manuscript-meta">
                    {move || format!(
                        "{} \u{00B7} {}",
                        lang.get().text_with("chapters_count", "n", &chapters.to_string()),
                        lang.get().text_with("chunks_count", "n", &chunks.to_string())
                    )}
                </span>
            </div>
            <span class="tag" data-status=status_for_tag>{move || lang.get().text(status_key(&status))}</span>
            {archivable.then(|| view! {
                <button class="btn btn-ghost" disabled=move || busy.get() on:click=archive>
                    {move || if armed.get() { lang.get().text("confirm_again") } else { lang.get().text("archive") }}
                </button>
            })}
        </li>
    }
}

#[component]
fn ManuscriptsPanel(job: RwSignal<Option<JobStatusDto>>) -> impl IntoView {
    let (lang, _) = use_lang();
    let toasts = use_toasts();
    let (filter, set_filter) = signal(None::<&'static str>);
    let (rows, set_rows) = signal(Vec::<AdminBookRowDto>::new());
    let (exhausted, set_exhausted) = signal(true);
    let (loading, set_loading) = signal(true);

    let load = move |reset: bool| {
        set_loading.set(true);
        let cursor = if reset {
            None
        } else {
            rows.get_untracked().last().map(|r| r.id)
        };
        let status = filter.get_untracked();
        spawn_local(async move {
            match api::admin_books(status, cursor).await {
                Ok(page) => {
                    set_exhausted.set(page.len() < api::ADMIN_PAGE_SIZE);
                    if reset {
                        set_rows.set(page);
                    } else {
                        set_rows.update(|all| all.extend(page));
                    }
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
            set_loading.set(false);
        });
    };
    load(true);

    let pick = move |status: Option<&'static str>| {
        set_filter.set(status);
        load(true);
    };
    let on_updated = Callback::new(move |updated: AdminBookRowDto| {
        set_rows.update(|all| {
            if let Some(row) = all.iter_mut().find(|r| r.id == updated.id) {
                *row = updated;
            }
        });
    });
    let filter_chip = move |status: Option<&'static str>, key: &'static str| {
        let active = move || filter.get() == status;
        view! {
            <button class="chip" class:active=active aria-pressed=active on:click=move |_| pick(status)>
                {move || lang.get().text(key)}
            </button>
        }
    };

    view! {
        <UploadCard job=job/>
        {move || job.get().map(|j| view! { <JobCard job=j/> })}
        <section aria-labelledby="manuscripts-title">
            <div class="section-head">
                <h2 id="manuscripts-title" class="section-title">{move || lang.get().text("tab_manuscripts")}</h2>
                <div class="chip-row" role="group" aria-label=move || lang.get().text("tab_manuscripts")>
                    {filter_chip(None, "chip_all")}
                    {STATUSES.iter().map(|(status, key)| filter_chip(Some(status), key)).collect::<Vec<_>>()}
                </div>
            </div>
            <ul class="manuscript-list">
                {move || rows.get().into_iter().map(|row| view! { <ManuscriptRow row=row on_updated=on_updated/> }).collect::<Vec<_>>()}
            </ul>
            {move || (!loading.get() && rows.get().is_empty()).then(|| view! {
                <p class="form-hint">{move || lang.get().text("no_manuscripts")}</p>
            })}
            {move || (!exhausted.get()).then(|| view! {
                <div class="load-more">
                    <button class="btn btn-ghost" on:click=move |_| load(false) disabled=move || loading.get()>
                        {move || if loading.get() { lang.get().text("loading") } else { lang.get().text("load_more") }}
                    </button>
                </div>
            })}
        </section>
    }
}

#[component]
fn QueuePanel(job: RwSignal<Option<JobStatusDto>>) -> impl IntoView {
    let (lang, _) = use_lang();
    let toasts = use_toasts();
    let (entries, set_entries) = signal(Vec::<DlqEntryDto>::new());
    let (loaded, set_loaded) = signal(false);
    spawn_local(async move {
        if let Ok(list) = api::admin_dlq().await {
            set_entries.set(list);
        }
        set_loaded.set(true);
    });
    let replay = move |entry_id: String| {
        spawn_local(async move {
            match api::admin_replay(&entry_id).await {
                Ok(_) => {
                    toasts.info(lang.get_untracked().text("replayed"));
                    set_entries.update(|all| all.retain(|e| e.id != entry_id));
                }
                Err(e) if e.contains("CONFLICT") => {
                    toasts.error(lang.get_untracked().text("replay_expired"));
                    set_entries.update(|all| all.retain(|x| x.id != entry_id));
                }
                Err(_) => toasts.error(lang.get_untracked().text("error_generic")),
            }
        });
    };
    view! {
        {move || job.get().map(|j| view! { <JobCard job=j/> })}
        <section aria-labelledby="dlq-title">
            <div class="section-head">
                <h2 id="dlq-title" class="section-title">{move || lang.get().text("dlq_title")}</h2>
                <span class="form-hint">{move || entries.get().len()}</span>
            </div>
            {move || (loaded.get() && entries.get().is_empty()).then(|| view! {
                <p class="form-hint">{move || lang.get().text("dlq_empty")}</p>
            })}
            <ul class="manuscript-list">
                {move || entries.get().into_iter().map(|entry| {
                    let short_id = entry.job_id.chars().take(8).collect::<String>();
                    let id = entry.id.clone();
                    view! {
                        <li class="dlq-row">
                            <div>
                                <span class="kicker">{move || format!("{} {short_id}", lang.get().text("admin_job"))}</span>
                                <p class="dlq-error">{entry.error.clone()}</p>
                                <small class="manuscript-meta">{move || lang.get().date_short(entry.failed_at)}</small>
                            </div>
                            <button class="btn btn-ghost" on:click=move |_| replay(id.clone())>{icons::recap()}<span>{move || lang.get().text("replay")}</span></button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </section>
    }
}

#[component]
fn RetentionPanel() -> impl IntoView {
    let (lang, _) = use_lang();
    let (books, set_books) = signal(Vec::<AdminBookRowDto>::new());
    let (selected, set_selected) = signal(None::<uuid::Uuid>);
    let (funnel, set_funnel) = signal(Vec::<DropOffPointDto>::new());
    spawn_local(async move {
        if let Ok(list) = api::admin_books(Some("published"), None).await {
            set_selected.set(list.first().map(|b| b.id));
            set_books.set(list);
        }
    });
    Effect::new(move || {
        let Some(id) = selected.get() else {
            return;
        };
        spawn_local(async move {
            if let Ok(points) = api::admin_dropoff(id).await {
                set_funnel.set(points);
            }
        });
    });
    let first_reach = move || {
        funnel
            .get()
            .first()
            .map(|p| p.readers_reached)
            .unwrap_or(0)
            .max(1)
    };
    view! {
        <section aria-labelledby="retention-title" class="retention">
            <div class="section-head">
                <h2 id="retention-title" class="section-title">{move || lang.get().text("tab_retention")}</h2>
                <span class="form-hint">{move || lang.get().text("funnel_hint")}</span>
            </div>
            <label class="field">
                <span>{move || lang.get().text("funnel_pick")}</span>
                <select class="input" on:change=move |ev| set_selected.set(event_target_value(&ev).parse().ok())>
                    {move || books.get().into_iter().map(|b| {
                        let is_selected = selected.get() == Some(b.id);
                        view! { <option value=b.id.to_string() selected=is_selected>{format!("{} \u{2014} {}", b.title, b.author)}</option> }
                    }).collect::<Vec<_>>()}
                </select>
            </label>
            {move || (selected.get().is_some() && funnel.get().is_empty()).then(|| view! {
                <p class="form-hint">{move || lang.get().text("no_funnel")}</p>
            })}
            <ol class="funnel">
                {move || funnel.get().into_iter().map(|p| {
                    let width = (p.readers_reached as f32 / first_reach() as f32 * 100.0).clamp(0.0, 100.0);
                    view! {
                        <li class="funnel-row">
                            <span class="funnel-label">{format!("{} \u{00B7} {}", roman(p.chapter_number), p.chapter_title)}</span>
                            <div class="funnel-bar" role="img" aria-label=move || lang.get().text_with("readers_reached", "n", &p.readers_reached.to_string())>
                                <span style=format!("width: {width:.1}%")></span>
                            </div>
                            <span class="funnel-meta">
                                {move || format!(
                                    "{} \u{00B7} {}",
                                    lang.get().text_with("readers_reached", "n", &p.readers_reached.to_string()),
                                    lang.get().text_with("drop_off", "n", &format!("{:.0}", p.drop_off_pct))
                                )}
                            </span>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ol>
        </section>
    }
}

/// The three curator panels behind one tab strip; rendered without the
/// role guard by the component gallery.
#[component]
pub fn CuratorDesk() -> impl IntoView {
    let (lang, _) = use_lang();
    let (desk, set_desk) = signal(Desk::Manuscripts);
    let job = RwSignal::new(None::<JobStatusDto>);
    let tab = move |which: Desk, key: &'static str| {
        let active = move || desk.get() == which;
        view! {
            <button role="tab" aria-selected=active class:active=active on:click=move |_| set_desk.set(which)>
                {move || lang.get().text(key)}
            </button>
        }
    };
    view! {
        <div class="desk">
            <div>
                <div class="kicker">{move || lang.get().text("admin_title")}</div>
                <h1 class="section-title">{move || lang.get().text("admin_desk")}</h1>
            </div>
            <div class="segmented desk-tabs" role="tablist" aria-label=move || lang.get().text("admin_desk")>
                {tab(Desk::Manuscripts, "tab_manuscripts")}
                {tab(Desk::Queue, "tab_queue")}
                {tab(Desk::Retention, "tab_retention")}
            </div>
            {move || match desk.get() {
                Desk::Manuscripts => view! { <ManuscriptsPanel job=job/> }.into_any(),
                Desk::Queue => view! { <QueuePanel job=job/> }.into_any(),
                Desk::Retention => view! { <RetentionPanel/> }.into_any(),
            }}
        </div>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let (lang, _) = use_lang();
    let session = use_session();
    view! {
        <div class="container">
            {move || match session.profile.get() {
                None if session.authed.get() => view! { <div class="skeleton streak-skeleton"></div> }.into_any(),
                Some(p) if p.role == "admin" => view! { <CuratorDesk/> }.into_any(),
                _ => view! {
                    <div class="empty-state">
                        <img src="/assets/admin-sorting-pigeonholes.webp" alt="" width="640" height="640"/>
                        <p>{move || lang.get().text("admin_forbidden")}</p>
                        <a href="/" class="btn btn-ghost">{move || lang.get().text("back_to_catalog")}</a>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
