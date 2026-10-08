//! In-page anchors. The router scrolls to a hash once, right as the new
//! view mounts, before async content has laid out; pages call back in here
//! after loading so the target ends up where the reader expects it.

use web_sys::{ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

pub fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|m| m.matches())
        .unwrap_or(false)
}

/// Brings an element to the top of the viewport, smoothly unless the user
/// asked for reduced motion. `scroll-margin-top` on the element keeps it
/// clear of the sticky header.
pub fn reveal(el: &web_sys::Element) {
    let options = ScrollIntoViewOptions::new();
    options.set_behavior(if prefers_reduced_motion() {
        ScrollBehavior::Auto
    } else {
        ScrollBehavior::Smooth
    });
    options.set_block(ScrollLogicalPosition::Start);
    el.scroll_into_view_with_scroll_into_view_options(&options);
}

pub fn scroll_to_hash(hash: &str) {
    let id = hash.trim_start_matches('#');
    if id.is_empty() {
        return;
    }
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        reveal(&el);
    }
}

pub fn scroll_to_location_hash() {
    if let Some(hash) = web_sys::window().and_then(|w| w.location().hash().ok()) {
        scroll_to_hash(&hash);
    }
}
