//! Two-step confirmation for destructive buttons.

use leptos::{html, prelude::*};
use std::time::Duration;
use wasm_bindgen::JsCast;

/// How long an armed button waits for its second press.
pub const CONFIRM_WINDOW: Duration = Duration::from_secs(4);

/// The first press arms a destructive button and the second fires it. The
/// arm lapses after [`CONFIRM_WINDOW`] or at a click anywhere else, so a
/// stray tap minutes later cannot fire it.
#[derive(Clone, Copy)]
pub struct Confirm {
    pub armed: ReadSignal<bool>,
    pub node: NodeRef<html::Button>,
    set_armed: WriteSignal<bool>,
    generation: StoredValue<u32>,
}

pub fn use_confirm() -> Confirm {
    let (armed, set_armed) = signal(false);
    let node = NodeRef::<html::Button>::new();
    let outside = window_event_listener(leptos::ev::click, move |ev| {
        if !armed.get_untracked() {
            return;
        }
        let target = ev.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        let inside = node
            .get_untracked()
            .zip(target)
            .is_some_and(|(button, target)| button.contains(Some(&target)));
        if !inside {
            set_armed.set(false);
        }
    });
    on_cleanup(move || outside.remove());
    Confirm {
        armed,
        node,
        set_armed,
        generation: StoredValue::new(0),
    }
}

impl Confirm {
    /// True on the press that confirms; otherwise arms and returns false.
    pub fn press(&self) -> bool {
        if self.armed.get_untracked() {
            self.set_armed.set(false);
            return true;
        }
        self.set_armed.set(true);
        // Only the latest arm may lapse it: an older timer would disarm a
        // button the reader armed again a moment ago.
        let generation = self.generation.get_value().wrapping_add(1);
        self.generation.set_value(generation);
        let (set_armed, current) = (self.set_armed, self.generation);
        set_timeout(
            move || {
                if current.try_get_value() == Some(generation) {
                    set_armed.try_set(false);
                }
            },
            CONFIRM_WINDOW,
        );
        false
    }
}
