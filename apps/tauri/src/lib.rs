//! Tauri shell around the Rotaria web bundle: the same WebAssembly app as the
//! website, told where the API lives through `window.__ROTARIA_API__`, with
//! `rotaria://book/<id>` links opening that book.

use std::sync::{Arc, Mutex};
use tauri::webview::PageLoadEvent;
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_deep_link::DeepLinkExt;

/// API base the app talks to, fixed when the app is built
/// (`ROTARIA_API_BASE`; an Android emulator reaches the host at 10.0.2.2).
const API_BASE: &str = match option_env!("ROTARIA_API_BASE") {
    Some(base) => base,
    None => "http://localhost:8080/api/v1",
};

/// In-app path for a `rotaria://book/<uuid>` link; anything else is ignored,
/// and the id must parse as a UUID before it reaches the page.
fn route_for(url: &Url) -> Option<String> {
    if url.scheme() != "rotaria" || url.host_str() != Some("book") {
        return None;
    }
    let id = uuid::Uuid::parse_str(url.path().trim_matches('/')).ok()?;
    Some(format!("/book/{id}"))
}

/// Book links waiting for the first page load. On a cold start Android
/// hands the link over only after the window exists and before its page has
/// loaded; navigating then would be lost to the load that follows.
#[derive(Default, Clone)]
struct LinkGate(Arc<Mutex<GateState>>);

#[derive(Default)]
struct GateState {
    loaded: bool,
    waiting: Option<String>,
}

impl LinkGate {
    /// The path to open now, or `None` when it must wait for the page.
    fn offer(&self, path: String) -> Option<String> {
        let mut state = self.0.lock().ok()?;
        if state.loaded {
            Some(path)
        } else {
            state.waiting = Some(path);
            None
        }
    }

    /// Marks the page loaded and hands back a link that waited for it.
    fn page_loaded(&self) -> Option<String> {
        let mut state = self.0.lock().ok()?;
        state.loaded = true;
        state.waiting.take()
    }
}

fn navigate(window: &tauri::WebviewWindow, path: &str) {
    let _ = window.eval(format!("window.location.assign('{path}')"));
}

fn build(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let gate = LinkGate::default();
    if let Some(path) = app
        .deep_link()
        .get_current()
        .ok()
        .flatten()
        .and_then(|urls| urls.iter().find_map(route_for))
    {
        gate.offer(path);
    }
    let on_load = gate.clone();
    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
        .title("Rotaria")
        .inner_size(1100.0, 760.0)
        .min_inner_size(360.0, 560.0)
        .initialization_script(format!(
            "window.__ROTARIA_API__ = {};",
            serde_json::to_string(API_BASE)?
        ))
        .on_page_load(move |window, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished) {
                if let Some(path) = on_load.page_loaded() {
                    navigate(&window, &path);
                }
            }
        })
        .build()?;
    // Desktop builds register the scheme for this user at run time; mobile
    // declares it in the manifest generated from tauri.conf.json.
    #[cfg(any(target_os = "linux", windows))]
    app.deep_link().register_all()?;
    app.deep_link().on_open_url(move |event| {
        if let Some(path) = event
            .urls()
            .iter()
            .find_map(route_for)
            .and_then(|p| gate.offer(p))
        {
            navigate(&window, &path);
        }
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let outcome = tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .setup(build)
        .run(tauri::generate_context!());
    if let Err(error) = outcome {
        eprintln!("Rotaria could not start: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(link: &str) -> Option<String> {
        route_for(&Url::parse(link).ok()?)
    }

    #[test]
    fn links_before_the_first_load_wait_for_it() {
        let gate = LinkGate::default();
        assert_eq!(gate.offer("/book/a".to_string()), None, "no page yet");
        assert_eq!(gate.page_loaded().as_deref(), Some("/book/a"));
        assert_eq!(gate.page_loaded(), None, "handed over once");
        assert_eq!(
            gate.offer("/book/b".to_string()).as_deref(),
            Some("/book/b")
        );
    }

    #[test]
    fn book_links_open_the_book_and_nothing_else() {
        assert_eq!(
            route("rotaria://book/a0000000-0000-4000-8000-000000000001").as_deref(),
            Some("/book/a0000000-0000-4000-8000-000000000001")
        );
        assert_eq!(route("rotaria://book/not-a-uuid"), None);
        assert_eq!(route("rotaria://book/x');alert(1)//"), None);
        assert_eq!(
            route("rotaria://read/a0000000-0000-4000-8000-000000000001"),
            None
        );
        assert_eq!(
            route("https://book/a0000000-0000-4000-8000-000000000001"),
            None
        );
    }
}
