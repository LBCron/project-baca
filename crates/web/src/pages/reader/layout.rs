//! Column geometry for the paginated reader: the chapter flows into CSS
//! columns one page wide, so page arithmetic is layout arithmetic.

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

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

pub fn columns_for(width: i32) -> usize {
    if width >= TWO_COLUMN_MIN_WIDTH {
        2
    } else {
        1
    }
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
