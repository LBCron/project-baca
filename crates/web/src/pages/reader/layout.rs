//! Column geometry for the paginated reader: the chapter flows into CSS
//! columns one page wide, so page arithmetic is layout arithmetic.

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// Window width from which the reader opens as a two-page spread; the same
/// number as the desktop breakpoint in `main.css`, and measured on the
/// window, not the frame, whose padding would otherwise push the spread out
/// to 1144 px windows.
pub const TWO_COLUMN_MIN_WIDTH: i32 = 1024;

#[derive(Debug, Clone, Copy, Default)]
pub struct Paddings {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
}

fn css_px(el: &HtmlElement, property: &str) -> f64 {
    web_sys::window()
        .and_then(|w| w.get_computed_style(el).ok().flatten())
        .and_then(|s| s.get_property_value(property).ok())
        .and_then(|v| v.trim_end_matches("px").parse::<f64>().ok())
        .unwrap_or(0.0)
}

pub fn paddings(el: &HtmlElement) -> Paddings {
    Paddings {
        top: css_px(el, "padding-top"),
        bottom: css_px(el, "padding-bottom"),
        left: css_px(el, "padding-left"),
    }
}

pub fn columns_for(window_width: i32) -> usize {
    if window_width >= TWO_COLUMN_MIN_WIDTH || spans_two_screens() {
        2
    } else {
        1
    }
}

/// A window spread across two side-by-side screens (a hinged dual-screen
/// device). A single column would run through the hinge; a spread puts the
/// spine on it, and `main.css` widens the gap to at least the hinge.
fn spans_two_screens() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(horizontal-viewport-segments: 2)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

pub fn window_width() -> i32 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|w| w.as_f64())
        .map(|w| w as i32)
        .unwrap_or(0)
}

pub fn paragraphs(el: &HtmlElement) -> Vec<HtmlElement> {
    let Ok(nodes) = el.query_selector_all(".chapter-body p") else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|i| nodes.item(i))
        .filter_map(|n| n.dyn_into::<HtmlElement>().ok())
        .collect()
}

/// Page index of an element from its layout offset; one page stride is the
/// viewport width because the column gap equals twice the side padding.
pub fn page_of(offset_left: i32, stride: i32) -> usize {
    if stride <= 0 {
        0
    } else {
        (offset_left / stride).max(0) as usize
    }
}

pub fn anchor_index(cfi: &str) -> Option<usize> {
    cfi.strip_prefix("p-")
        .and_then(|n| n.parse::<usize>().ok())
        .map(|n| n.saturating_sub(1))
}
