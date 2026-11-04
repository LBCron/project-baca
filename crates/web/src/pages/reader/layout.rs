//! Column geometry for the paginated reader: the chapter flows into CSS
//! columns one page wide, so page arithmetic is layout arithmetic.

use leptos::prelude::on_cleanup;
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

/// Calls `on_change` whenever the element's box changes size, until the
/// owner is cleaned up.
pub fn watch_size(el: &HtmlElement, on_change: impl Fn() + 'static) {
    let callback = wasm_bindgen::closure::Closure::<dyn Fn()>::new(on_change);
    let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) else {
        return;
    };
    observer.observe(el);
    let attached = send_wrapper::SendWrapper::new((observer, callback));
    on_cleanup(move || {
        let (observer, _callback) = attached.take();
        observer.disconnect();
    });
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

/// Where reading stands: a paragraph and the share of its text laid out
/// before the page. Unlike a `p-N` anchor it survives a relayout that moves
/// the page break into the middle of that paragraph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    pub index: usize,
    pub fraction: f64,
}

/// (page, height) of each column fragment a paragraph is split into.
fn fragments(viewport: &HtmlElement, paragraph: &HtmlElement) -> Vec<(usize, f64)> {
    let stride = viewport.client_width();
    let origin = viewport.get_bounding_client_rect().left() - f64::from(viewport.scroll_left());
    let rects = paragraph.get_client_rects();
    (0..rects.length())
        .filter_map(|i| rects.get(i))
        .filter(|r| r.width() > 0.0)
        .map(|r| {
            (
                page_of((r.left() - origin).round() as i32, stride),
                r.height(),
            )
        })
        .collect()
}

/// The spot at the top of `page`: the first paragraph with text on it.
pub fn spot_on(viewport: &HtmlElement, page: usize) -> Option<Spot> {
    paragraphs(viewport)
        .iter()
        .enumerate()
        .find_map(|(index, p)| {
            let parts = fragments(viewport, p);
            parts.iter().any(|(at, _)| *at >= page).then(|| {
                let total: f64 = parts.iter().map(|(_, h)| h).sum();
                let before: f64 = parts
                    .iter()
                    .filter(|(at, _)| *at < page)
                    .map(|(_, h)| h)
                    .sum();
                Spot {
                    index,
                    fraction: if total > 0.0 { before / total } else { 0.0 },
                }
            })
        })
}

/// The page that holds `spot` in the current layout.
pub fn page_of_spot(viewport: &HtmlElement, spot: Spot) -> Option<usize> {
    let paragraph = paragraphs(viewport).into_iter().nth(spot.index)?;
    let parts = fragments(viewport, &paragraph);
    let total: f64 = parts.iter().map(|(_, h)| h).sum();
    fragment_at(&parts, spot.fraction * total)
}

/// Page of the fragment that holds the text `offset` pixels into the
/// paragraph; a fragment ending exactly there hands over to the next one.
fn fragment_at(parts: &[(usize, f64)], offset: f64) -> Option<usize> {
    let mut seen = 0.0;
    parts
        .iter()
        .find(|(_, h)| {
            seen += h;
            seen - offset > 0.5
        })
        .or(parts.last())
        .map(|(page, _)| *page)
}

pub fn anchor_index(cfi: &str) -> Option<usize> {
    cfi.strip_prefix("p-")
        .and_then(|n| n.parse::<usize>().ok())
        .map(|n| n.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::fragment_at;

    #[test]
    fn spot_lands_on_the_fragment_that_holds_it() {
        let parts = [(3, 100.0), (4, 300.0), (5, 100.0)];
        assert_eq!(fragment_at(&parts, 0.0), Some(3));
        assert_eq!(
            fragment_at(&parts, 100.0),
            Some(4),
            "a break at the offset hands over"
        );
        assert_eq!(fragment_at(&parts, 399.7), Some(5));
        assert_eq!(fragment_at(&parts, 1e9), Some(5));
        assert_eq!(fragment_at(&[], 0.0), None);
    }
}
