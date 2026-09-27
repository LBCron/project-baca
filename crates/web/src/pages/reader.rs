//! Reflowable reader with CFI position tracking.

use crate::api;
use crate::components::insights::CatchupRecap;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared::ChapterDetailDto;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

fn query_param(name: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    search.trim_start_matches('?').split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(k), Some(v)) if k == name => Some(v.to_string()),
            _ => None,
        }
    })
}

#[component]
pub fn ReaderPage() -> impl IntoView {
    let params = use_params_map();
    let book_id = move || params.get().get("id").unwrap_or_default();
    let start_chapter: i32 = query_param("chapter")
        .and_then(|c| c.parse().ok())
        .unwrap_or(1);

    let (chapter, set_chapter) = signal(None::<ChapterDetailDto>);
    let (chapter_no, set_chapter_no) = signal(start_chapter);
    let (error, set_error) = signal(None::<String>);
    let (menu_open, set_menu_open) = signal(false);
    let (font_size, set_font_size) = signal(18);
    let (line_height, set_line_height) = signal(17);
    let (anchor, set_anchor) = signal(String::from("top"));
    let (streak, set_streak) = signal(None::<i32>);
    let recap_open = RwSignal::new(false);

    let load = move |book: String, number: i32| {
        spawn_local(async move {
            match api::chapter(&book, number).await {
                Ok(detail) => {
                    set_chapter.set(Some(detail));
                    set_error.set(None);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    {
        let id = book_id();
        load(id, start_chapter);
    }

    let next = move |_| {
        let number = chapter_no.get_untracked() + 1;
        set_chapter_no.set(number);
        set_anchor.set(String::from("top"));
        load(book_id(), number);
    };
    let prev = move |_| {
        let number = (chapter_no.get_untracked() - 1).max(1);
        set_chapter_no.set(number);
        set_anchor.set(String::from("top"));
        load(book_id(), number);
    };
    let toggle_menu = move |_| set_menu_open.update(|m| *m = !*m);

    let viewport_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move || {
        chapter.get();
        let Some(div) = viewport_ref.get() else {
            return;
        };
        div.set_scroll_left(0);
        let document = match api::window_document() {
            Some(d) => d,
            None => return,
        };
        let paragraphs = document.get_elements_by_tag_name("p");
        let len = paragraphs.length();
        let mut i = 0;
        while i < len {
            if let Some(el) = paragraphs.item(i) {
                let idx = i + 1;
                let _ = el.set_attribute("data-cfi", &format!("p-{idx}"));
            }
            i += 1;
        }
        let observer_cb =
            Closure::<dyn Fn(js_sys::Array)>::new(Box::new(move |entries: js_sys::Array| {
                let mut j = 0;
                while j < entries.length() {
                    if let Ok(entry) = entries
                        .get(j)
                        .dyn_into::<web_sys::IntersectionObserverEntry>()
                    {
                        if entry.is_intersecting() {
                            if let Some(target) = entry.target().dyn_ref::<web_sys::Element>() {
                                if let Some(cfi) = target.get_attribute("data-cfi") {
                                    set_anchor.set(cfi);
                                }
                            }
                        }
                    }
                    j += 1;
                }
            }) as Box<dyn Fn(js_sys::Array)>);
        let observer =
            web_sys::IntersectionObserver::new(observer_cb.as_ref().unchecked_ref()).ok();
        observer_cb.forget();
        if let Some(obs) = observer {
            let mut k = 0;
            while k < len {
                if let Some(el) = paragraphs.item(k) {
                    obs.observe(&el);
                }
                k += 1;
            }
            obs.disconnect();
        }
    });

    let persist_progress = move |_| {
        let id = book_id();
        let number = chapter_no.get_untracked();
        let cfi = anchor.get_untracked();
        spawn_local(async move {
            if let Ok(detail) = api::chapter(&id, number).await {
                let update = shared::ReadingProgressUpdateDto {
                    chapter_id: detail.id,
                    last_anchor_cfi: cfi,
                    completion_percentage: 0.0,
                };
                let _ = api::save_progress(&id, &update).await;
                if let Ok(uuid) = id.parse::<uuid::Uuid>() {
                    if let Ok(resp) = api::heartbeat(&shared::ReadingHeartbeatRequest {
                        book_id: uuid,
                        seconds_spent: 300,
                    })
                    .await
                    {
                        if resp.streak_incremented {
                            set_streak.set(Some(resp.current_streak_days));
                        }
                    }
                }
            }
        });
    };

    view! {
        <div class="reader-shell">
            <header class="reader-bar">
                <a href=format!("/book/{}", book_id()) class="nav-link">"‹ Catalog"</a>
                <span class="reader-title">
                    {move || chapter.get().map(|c| format!("Chapter {} — {}", c.chapter_number, c.title)).unwrap_or_default()}
                </span>
                <div class="reader-tools">
                    {move || (chapter_no.get() > 1).then(|| view! {
                        <button class="lang-switch" on:click=move |_| recap_open.set(true)>"Recap"</button>
                    })}
                    <button class="lang-switch" on:click=toggle_menu>"Aa"</button>
                </div>
            </header>
            <CatchupRecap book_id=book_id() chapter=chapter_no.get() show=recap_open/>

            {move || menu_open.get().then(|| view! {
                <aside class="type-drawer">
                    <label>
                        "Font size: " {move || font_size.get()} "px"
                        <input type="range" min="14" max="22" prop:value=move || font_size.get()
                            on:input=move |ev| set_font_size.set(event_target_value(&ev).parse().unwrap_or(18))/>
                    </label>
                    <label>
                        "Line height: " {move || format!("{:.1}", line_height.get() as f32 / 10.0)}
                        <input type="range" min="15" max="19" prop:value=move || line_height.get()
                            on:input=move |ev| set_line_height.set(event_target_value(&ev).parse().unwrap_or(17))/>
                    </label>
                </aside>
            })}

            <div
                node_ref=viewport_ref
                class="reader-viewport"
                style=move || format!("--reader-font-size: {}px; --reader-line-height: {:.1};", font_size.get(), line_height.get() as f32 / 10.0)
            >
                {move || match chapter.get() {
                    None => match error.get() {
                        None => view! { <p>"Loading chapter…"</p> }.into_any(),
                        Some(e) => view! { <p>{format!("Failed: {e}")}</p> }.into_any(),
                    },
                    Some(c) => view! { <div class="chapter-body" inner_html=c.html_content></div> }.into_any(),
                }}
            </div>

            <div class="reader-zone reader-zone-prev" on:click=prev></div>
            <div class="reader-zone reader-zone-menu" on:click=toggle_menu></div>
            <div class="reader-zone reader-zone-next" on:click=next></div>

            <footer class="reader-status" on:click=persist_progress>
                {move || format!("Anchor: {} (tap to save position)", anchor.get())}
            </footer>
            {move || streak.get().map(|days| view! {
                <div class="streak-toast" role="status">{format!("❖ {days}-day streak")}</div>
            })}
        </div>
    }
}
