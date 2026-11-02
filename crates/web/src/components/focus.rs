//! Focus bookkeeping for sheets and dialogs: remember the control that
//! opened them and hand focus back when they close.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Hides a sheet once the event being handled has finished. A click inside
/// a sheet still bubbles to the sheet's and the backdrop's own listeners;
/// unmounting the sheet before then frees those closures, and the browser
/// then calls freed handlers. Microtasks run between listeners of one
/// dispatch, so only a task (a zero timeout) waits for the whole of it.
pub fn close_after_event(show: RwSignal<bool>) {
    set_timeout(
        move || {
            show.try_set(false);
        },
        std::time::Duration::ZERO,
    );
}

pub fn active_element() -> Option<web_sys::HtmlElement> {
    web_sys::window()?
        .document()?
        .active_element()?
        .dyn_into::<web_sys::HtmlElement>()
        .ok()
}

const FOCUSABLE: &str = "a[href], button:not([disabled]), input:not([disabled]), \
     select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";

/// Focusable elements of the dialog that holds `inside`, in document order.
fn dialog_focusables(inside: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    let Some(dialog) = inside.closest("[role='dialog']").ok().flatten() else {
        return Vec::new();
    };
    let Ok(nodes) = dialog.query_selector_all(FOCUSABLE) else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|i| nodes.item(i))
        .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
        .collect()
}

/// Keeps Tab and Shift+Tab cycling inside an open modal sheet: the page
/// behind the backdrop is out of reach for a keyboard, as it is for a pointer.
fn trap_tab(ev: &web_sys::KeyboardEvent, inside: &web_sys::Element) {
    let items = dialog_focusables(inside);
    let (Some(first), Some(last)) = (items.first(), items.last()) else {
        return;
    };
    let active = active_element();
    let within = active
        .as_ref()
        .is_some_and(|a| items.iter().any(|i| i == a));
    let target = match (ev.shift_key(), active) {
        (_, _) if !within => Some(first),
        (true, Some(a)) if &a == first => Some(last),
        (false, Some(a)) if &a == last => Some(first),
        _ => None,
    };
    if let Some(target) = target {
        ev.prevent_default();
        let _ = target.focus();
    }
}

/// The control a sheet focuses when it opens: the one marked
/// `data-autofocus`, otherwise the close button.
fn initial_focus(close: &web_sys::HtmlElement) -> web_sys::HtmlElement {
    close
        .closest("[role='dialog']")
        .ok()
        .flatten()
        .and_then(|dialog| dialog.query_selector("[data-autofocus]").ok().flatten())
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        .unwrap_or_else(|| close.clone())
}

fn focus_is_inside_dialog_of(el: &web_sys::Element) -> bool {
    let dialog = el.closest("[role='dialog']").ok().flatten();
    match (dialog, active_element()) {
        (Some(dialog), Some(active)) => dialog.contains(Some(&active)),
        _ => false,
    }
}

/// Moves focus into the sheet when it opens and restores it when it closes;
/// Tab stays inside it and Escape closes it. `close_button` takes focus
/// unless the sheet marks a control `data-autofocus`. `on_close` is the
/// sheet's own close routine so any extra reset (errors, busy flags) runs
/// the same way as the close button.
pub fn manage_sheet_focus(
    show: Signal<bool>,
    close_button: NodeRef<leptos::html::Button>,
    on_close: impl Fn() + Copy + 'static,
) {
    let opener = StoredValue::new(None::<web_sys::HtmlElement>);
    Effect::new(move || {
        if show.get() {
            opener.set_value(active_element());
            request_animation_frame(move || {
                let Some(button) = close_button.try_get_untracked().flatten() else {
                    return;
                };
                // On a slow device the frame can arrive after the reader has
                // already clicked or typed inside the sheet; moving focus then
                // would send their next Enter to another control.
                if !focus_is_inside_dialog_of(&button) {
                    let _ = initial_focus(&button).focus();
                }
            });
        } else if let Some(el) = opener.try_get_value().flatten() {
            let _ = el.focus();
            opener.set_value(None);
        }
    });
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if show.try_get_untracked() != Some(true) {
            return;
        }
        match ev.key().as_str() {
            "Escape" => on_close(),
            "Tab" => {
                if let Some(button) = close_button.try_get_untracked().flatten() {
                    trap_tab(&ev, &button);
                }
            }
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());
}
