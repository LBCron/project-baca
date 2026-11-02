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

/// Moves focus into the sheet when it opens and restores it when it closes;
/// Tab stays inside it and Escape closes it. `on_close` is the sheet's own
/// close routine so any extra reset (errors, busy flags) runs the same way
/// as the close button.
pub fn manage_sheet_focus(
    show: Signal<bool>,
    first_focus: NodeRef<leptos::html::Button>,
    on_close: impl Fn() + Copy + 'static,
) {
    let opener = StoredValue::new(None::<web_sys::HtmlElement>);
    Effect::new(move || {
        if show.get() {
            opener.set_value(active_element());
            request_animation_frame(move || {
                if let Some(button) = first_focus.try_get_untracked().flatten() {
                    let _ = button.focus();
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
                if let Some(button) = first_focus.try_get_untracked().flatten() {
                    trap_tab(&ev, &button);
                }
            }
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());
}
