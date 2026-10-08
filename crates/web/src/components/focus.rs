//! Focus bookkeeping for sheets and dialogs: remember the control that
//! opened them and hand focus back when they close.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub fn active_element() -> Option<web_sys::HtmlElement> {
    web_sys::window()?
        .document()?
        .active_element()?
        .dyn_into::<web_sys::HtmlElement>()
        .ok()
}

/// Moves focus into the sheet when it opens and restores it when it closes;
/// Escape closes it. `on_close` is the sheet's own close routine so any
/// extra reset (errors, busy flags) runs the same way as the close button.
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
        if ev.key() == "Escape" && show.try_get_untracked() == Some(true) {
            on_close();
        }
    });
    on_cleanup(move || keys.remove());
}
