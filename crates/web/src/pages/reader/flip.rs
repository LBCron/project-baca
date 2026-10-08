//! Paper page turn. The leaving page becomes a few vertical strips hinged on
//! the spine; each strip is a flat quad whose rotation lags its neighbour so
//! the sheet bends like paper, and a damped spring drives the angle from the
//! pointer or from a tap. Every strip holds a clone of the column flow that
//! reproduces exactly one column (a spacer stands in for the columns before
//! it), so building a turn costs a page of layout, not a chapter.

use super::layout::Paddings;
use leptos::prelude::{document, request_animation_frame};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Forward,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Single,
    Spread,
}

#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub stride: f64,
    pub height: f64,
    pub mode: Mode,
    pub pad: Paddings,
}

impl Geometry {
    pub fn measure(viewport: &HtmlElement, mode: Mode) -> Self {
        Self {
            stride: f64::from(viewport.client_width()),
            height: f64::from(viewport.client_height()),
            mode,
            pad: super::layout::paddings(viewport),
        }
    }

    fn columns_per_page(&self) -> f64 {
        match self.mode {
            Mode::Single => 1.0,
            Mode::Spread => 2.0,
        }
    }

    fn pitch(&self) -> f64 {
        self.stride / self.columns_per_page()
    }

    fn column_height(&self) -> f64 {
        (self.height - self.pad.top - self.pad.bottom).max(1.0)
    }

    /// Content x of the page half that shows `column`.
    fn page_x(&self, column: usize) -> f64 {
        match self.mode {
            Mode::Single => column as f64 * self.stride,
            Mode::Spread => {
                (column / 2) as f64 * self.stride + (column % 2) as f64 * (self.stride / 2.0)
            }
        }
    }
}

/// A turn from `from_page`; `None` as the destination lifts the sheet a
/// little and lets it fall back (the end of the book, or the first page).
#[derive(Debug, Clone, Copy)]
pub struct TurnPlan {
    pub dir: Dir,
    pub from_page: usize,
    pub to_page: Option<usize>,
}

/// Where a drag started, so the sheet's free edge stays under the finger.
#[derive(Debug, Clone, Copy)]
pub struct DragStart {
    pub x: f64,
    pub book_left: f64,
    pub book_right: f64,
    pub touch: bool,
}

const STRIPS_SPREAD: usize = 8;
const STRIPS_SINGLE: usize = 6;
const BEND_MAX: f64 = 0.55;
const RESIST_ANGLE: f64 = 22.0;
const TAP_VELOCITY: f64 = 140.0;
const FLICK_VELOCITY: f64 = 160.0;
const SETTLE_ANGLE: f64 = 0.25;
const SETTLE_VELOCITY: f64 = 4.0;
const MAX_FRAME_SECS: f64 = 0.032;
const HAPTIC_MS: f64 = 8.0;

/// Spring constants (stiffness, damping ratio): tight while a finger leads
/// the sheet, softer and a little underdamped once released so the landing
/// has weight.
const DRAG_SPRING: (f64, f64) = (900.0, 1.0);
const FREE_SPRING: (f64, f64) = (240.0, 0.86);

struct Strip {
    el: HtmlElement,
    front_tint: HtmlElement,
    rear_tint: HtmlElement,
}

struct Leaf {
    plan: TurnPlan,
    angle: f64,
    vel: f64,
    target: f64,
    rest: f64,
    done: f64,
    dragging: bool,
    start_x: f64,
    span: f64,
    touch: bool,
    width: f64,
    hinge_right: bool,
    strips: Vec<Strip>,
    cast: HtmlElement,
}

impl Leaf {
    fn resisting(&self) -> bool {
        self.plan.to_page.is_none()
    }

    fn rises(&self) -> bool {
        self.rest < self.done
    }

    /// The sheet's lift above the page, whatever side it rests on.
    fn lift(&self) -> f64 {
        if self.rises() {
            self.angle
        } else {
            180.0 - self.angle
        }
    }
}

struct Inner {
    layer: HtmlElement,
    viewport: HtmlElement,
    on_settle: Rc<dyn Fn(usize)>,
    leaf: Option<Leaf>,
    looping: bool,
    last_tick: f64,
}

#[derive(Clone)]
pub struct Flipper {
    inner: Rc<RefCell<Inner>>,
}

impl Flipper {
    pub fn new(
        layer: HtmlElement,
        viewport: HtmlElement,
        on_settle: impl Fn(usize) + 'static,
    ) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                layer,
                viewport,
                on_settle: Rc::new(on_settle),
                leaf: None,
                looping: false,
                last_tick: 0.0,
            })),
        }
    }

    pub fn is_active(&self) -> bool {
        self.inner.borrow().leaf.is_some()
    }

    /// Starts a turn. Returns `false` when another turn is running or the
    /// page cannot be rebuilt, so the caller turns the page without paper.
    pub fn begin(&self, plan: TurnPlan, geo: &Geometry, drag: Option<DragStart>) -> bool {
        {
            let inner = self.inner.borrow();
            if inner.leaf.is_some() {
                return false;
            }
        }
        let Some(leaf) = self.build(plan, geo, drag) else {
            return false;
        };
        {
            let mut inner = self.inner.borrow_mut();
            if let Some(to) = plan.to_page {
                let scroll_now = match (geo.mode, plan.dir) {
                    (Mode::Single, Dir::Back) => None,
                    _ => Some(to),
                };
                if let Some(page) = scroll_now {
                    inner
                        .viewport
                        .set_scroll_left((page as f64 * geo.stride) as i32);
                }
            }
            inner.leaf = Some(leaf);
            inner.last_tick = js_sys::Date::now();
        }
        self.tick();
        true
    }

    pub fn drag_to(&self, x: f64) {
        let mut inner = self.inner.borrow_mut();
        let Some(leaf) = inner.leaf.as_mut() else {
            return;
        };
        if !leaf.dragging {
            return;
        }
        let travel = match leaf.plan.dir {
            Dir::Forward => leaf.start_x - x,
            Dir::Back => x - leaf.start_x,
        };
        let lifted = angle_for_travel(travel, leaf.span);
        let lifted = if leaf.resisting() {
            lifted.min(RESIST_ANGLE)
        } else {
            lifted
        };
        leaf.target = if leaf.rises() { lifted } else { 180.0 - lifted };
        drop(inner);
        self.loop_once();
    }

    /// Finishes the turn when the sheet is past the middle or was flicked;
    /// otherwise it falls back where it came from.
    pub fn release(&self) {
        let mut inner = self.inner.borrow_mut();
        let Some(leaf) = inner.leaf.as_mut() else {
            return;
        };
        if !leaf.dragging {
            return;
        }
        leaf.dragging = false;
        let flick = if leaf.rises() {
            leaf.vel > FLICK_VELOCITY
        } else {
            leaf.vel < -FLICK_VELOCITY
        };
        let complete = !leaf.resisting() && (leaf.lift() > 90.0 || flick);
        leaf.target = if complete { leaf.done } else { leaf.rest };
        drop(inner);
        self.loop_once();
    }

    /// Drops the sheet immediately and restores the page underneath.
    pub fn cancel(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(leaf) = inner.leaf.take() {
            let stride = f64::from(inner.viewport.client_width());
            inner
                .viewport
                .set_scroll_left((leaf.plan.from_page as f64 * stride) as i32);
        }
        clear_children(&inner.layer);
    }

    fn build(&self, plan: TurnPlan, geo: &Geometry, drag: Option<DragStart>) -> Option<Leaf> {
        let inner = self.inner.borrow();
        let doc = document();
        let viewport = &inner.viewport;
        let layer = &inner.layer;
        clear_children(layer);

        let half = geo.stride / 2.0;
        let from = plan.from_page;
        let (width, left, hinge_right, rest, done) = match (geo.mode, plan.dir) {
            (Mode::Spread, Dir::Forward) => (half, half, false, 0.0, 180.0),
            (Mode::Spread, Dir::Back) => (half, 0.0, true, 0.0, 180.0),
            (Mode::Single, Dir::Forward) => (geo.stride, 0.0, false, 0.0, 180.0),
            (Mode::Single, Dir::Back) => (geo.stride, 0.0, false, 180.0, 0.0),
        };
        let strips = match geo.mode {
            Mode::Spread => STRIPS_SPREAD,
            Mode::Single => STRIPS_SINGLE,
        };
        let w = width / strips as f64;

        // Columns shown by the sheet's two faces and by the half that must
        // keep showing the old page while the sheet travels.
        let (front_col, rear_col, hold) = match (geo.mode, plan.dir) {
            (Mode::Spread, Dir::Forward) => (
                Some(2 * from + 1),
                plan.to_page.map(|_| 2 * from + 2),
                Some((0.0, 2 * from)),
            ),
            (Mode::Spread, Dir::Back) => (
                Some(2 * from),
                (from > 0).then(|| 2 * from - 1),
                Some((half, 2 * from + 1)),
            ),
            (Mode::Single, Dir::Forward) => (Some(from), None, None),
            (Mode::Single, Dir::Back) => ((from > 0).then(|| from - 1), None, None),
        };
        if geo.mode == Mode::Single && plan.dir == Dir::Back && plan.to_page.is_none() {
            return None;
        }

        let front = front_col.and_then(|c| column_template(viewport, c, geo));
        let rear = rear_col.and_then(|c| column_template(viewport, c, geo));
        if front.is_none() && rear.is_none() {
            return None;
        }

        if let Some((hold_left, hold_col)) = hold {
            let keep = doc.create_element("div").ok()?;
            keep.set_class_name("flip-hold");
            set_style(&keep, &format!("left:{hold_left:.2}px;width:{half:.2}px"));
            match column_template(viewport, hold_col, geo) {
                Some(template) if plan.to_page.is_some() => {
                    let clone = template.clone_node_with_deep(true).ok()?;
                    let clone: HtmlElement = clone.dyn_into().ok()?;
                    place_clone(&clone, -geo.page_x(hold_col), geo.stride);
                    keep.append_child(&clone).ok()?;
                }
                _ => keep.class_list().add_1("blank").ok()?,
            }
            layer.append_child(&keep).ok()?;
        }

        let cast = doc.create_element("div").ok()?;
        cast.set_class_name(match (geo.mode, plan.dir) {
            (Mode::Spread, Dir::Forward) => "flip-cast left",
            (Mode::Spread, Dir::Back) => "flip-cast right",
            (Mode::Single, _) => "flip-cast full",
        });
        layer.append_child(&cast).ok()?;

        let leaf_el = doc.create_element("div").ok()?;
        leaf_el.set_class_name("flip-leaf");
        set_style(&leaf_el, &format!("left:{left:.2}px;width:{width:.2}px"));

        let mut strip_refs = Vec::with_capacity(strips);
        for j in 0..strips {
            let strip = doc.create_element("div").ok()?;
            strip.set_class_name("flip-strip");
            let place = if hinge_right {
                "right:0;transform-origin:right center"
            } else {
                "left:0;transform-origin:left center"
            };
            set_style(&strip, &format!("width:{:.2}px;{place}", w + 1.0));

            let near = j as f64 * w;
            let far = width - (j as f64 + 1.0) * w;
            let (front_clip, rear_clip) = match (geo.mode, plan.dir) {
                (Mode::Spread, Dir::Forward) => (near, far),
                (Mode::Spread, Dir::Back) => (far, near),
                (Mode::Single, _) => (near, near),
            };

            let (front_face, front_tint) = make_face(
                &doc,
                front.as_ref().map(|t| {
                    (
                        t,
                        front_col.map(|c| geo.page_x(c)).unwrap_or(0.0) + front_clip,
                    )
                }),
                geo.stride,
                false,
            )?;
            let (rear_face, rear_tint) = make_face(
                &doc,
                rear.as_ref().map(|t| {
                    (
                        t,
                        rear_col.map(|c| geo.page_x(c)).unwrap_or(0.0) + rear_clip,
                    )
                }),
                geo.stride,
                true,
            )?;
            strip.append_child(&front_face).ok()?;
            strip.append_child(&rear_face).ok()?;
            leaf_el.append_child(&strip).ok()?;
            strip_refs.push(Strip {
                el: strip.dyn_into().ok()?,
                front_tint,
                rear_tint,
            });
        }
        layer.append_child(&leaf_el).ok()?;

        let (dragging, start_x, span, touch) = match drag {
            Some(d) => {
                let span = match plan.dir {
                    Dir::Forward => d.x - d.book_left,
                    Dir::Back => d.book_right - d.x,
                };
                (true, d.x, span.max(1.0), d.touch)
            }
            None => (false, 0.0, 1.0, false),
        };
        let initial_velocity = if dragging {
            0.0
        } else if rest < done {
            TAP_VELOCITY
        } else {
            -TAP_VELOCITY
        };
        Some(Leaf {
            plan,
            angle: rest,
            vel: initial_velocity,
            target: if dragging { rest } else { done },
            rest,
            done,
            dragging,
            start_x,
            span,
            touch,
            width,
            hinge_right,
            strips: strip_refs,
            cast: cast.dyn_into().ok()?,
        })
    }

    fn loop_once(&self) {
        let start = {
            let mut inner = self.inner.borrow_mut();
            if inner.looping || inner.leaf.is_none() {
                false
            } else {
                inner.looping = true;
                inner.last_tick = js_sys::Date::now();
                true
            }
        };
        if start {
            let me = self.clone();
            request_animation_frame(move || me.step());
        }
    }

    fn tick(&self) {
        self.loop_once();
    }

    fn step(&self) {
        let outcome = {
            let mut inner = self.inner.borrow_mut();
            inner.looping = false;
            let now = js_sys::Date::now();
            let dt = ((now - inner.last_tick) / 1000.0).clamp(0.001, MAX_FRAME_SECS);
            inner.last_tick = now;
            let Some(leaf) = inner.leaf.as_mut() else {
                return;
            };
            let (k, zeta) = if leaf.dragging {
                DRAG_SPRING
            } else {
                FREE_SPRING
            };
            let omega = k.sqrt();
            let acc = k * (leaf.target - leaf.angle) - 2.0 * zeta * omega * leaf.vel;
            leaf.vel += acc * dt;
            leaf.angle += leaf.vel * dt;
            if leaf.angle <= 0.0 {
                leaf.angle = 0.0;
                leaf.vel = leaf.vel.max(0.0) * 0.25;
            }
            if leaf.angle >= 180.0 {
                leaf.angle = 180.0;
                leaf.vel = leaf.vel.min(0.0) * 0.25;
            }
            let settled = !leaf.dragging
                && (leaf.target - leaf.angle).abs() < SETTLE_ANGLE
                && leaf.vel.abs() < SETTLE_VELOCITY;
            if settled {
                let completed = leaf.target == leaf.done && !leaf.resisting();
                Some((completed, leaf.plan, leaf.touch))
            } else {
                let motion = if leaf.vel != 0.0 {
                    leaf.vel.signum()
                } else {
                    (leaf.target - leaf.angle).signum()
                };
                let speed = (0.35 + leaf.vel.abs() / 700.0).min(1.0);
                let bend = BEND_MAX * leaf.angle.to_radians().sin() * speed * motion;
                layout_strips(leaf, bend);
                None
            }
        };
        match outcome {
            None => {
                let me = self.clone();
                let mut inner = self.inner.borrow_mut();
                inner.looping = true;
                drop(inner);
                request_animation_frame(move || me.step());
            }
            Some((completed, plan, touch)) => self.settle(completed, plan, touch),
        }
    }

    fn settle(&self, completed: bool, plan: TurnPlan, touch: bool) {
        let on_settle = {
            let mut inner = self.inner.borrow_mut();
            inner.leaf = None;
            let stride = f64::from(inner.viewport.client_width());
            let page = if completed {
                plan.to_page.unwrap_or(plan.from_page)
            } else {
                plan.from_page
            };
            inner
                .viewport
                .set_scroll_left((page as f64 * stride) as i32);
            clear_children(&inner.layer);
            completed.then(|| (inner.on_settle.clone(), page))
        };
        if let Some((callback, page)) = on_settle {
            if touch {
                if let Some(window) = web_sys::window() {
                    let _ = window.navigator().vibrate_with_duration(HAPTIC_MS as u32);
                }
            }
            callback(page);
        }
    }
}

/// The sheet's free edge stays under the finger: a drag across the span
/// (finger to the far edge of the book) rotates it the full 180 degrees.
fn angle_for_travel(travel: f64, span: f64) -> f64 {
    let t = (travel / span.max(1.0)).clamp(0.0, 1.0);
    (1.0 - 2.0 * t).acos().to_degrees()
}

/// One flat quad per strip, chained from the hinge: the free edge leads
/// while the part near the spine lags, so the polyline reads as a curve.
fn layout_strips(leaf: &Leaf, bend: f64) {
    let n = leaf.strips.len().max(1) as f64;
    let w = leaf.width / n;
    let mut x = 0.0;
    let mut z = 0.0;
    for (j, strip) in leaf.strips.iter().enumerate() {
        let lag = (j as f64 + 0.5) / n - 0.5;
        let a = (leaf.angle * (1.0 + bend * lag)).clamp(0.0, 180.0);
        let (s, c) = a.to_radians().sin_cos();
        let transform = if leaf.hinge_right {
            format!("translate3d({:.2}px,0,{:.2}px) rotateY({:.3}deg)", -x, z, a)
        } else {
            format!("translate3d({:.2}px,0,{:.2}px) rotateY({:.3}deg)", x, z, -a)
        };
        let _ = strip.el.style().set_property("transform", &transform);
        let _ = strip
            .front_tint
            .style()
            .set_property("opacity", &format!("{:.3}", 0.34 * (1.0 - c) / 2.0));
        let _ = strip
            .rear_tint
            .style()
            .set_property("opacity", &format!("{:.3}", 0.34 * (1.0 + c) / 2.0));
        x += w * c;
        z += w * s;
    }
    let shadow = 0.7 * leaf.lift().to_radians().sin();
    let _ = leaf
        .cast
        .style()
        .set_property("opacity", &format!("{shadow:.3}"));
}

fn set_style(el: &Element, css: &str) {
    let _ = el.set_attribute("style", css);
}

/// Offsets a viewport clone without touching the inline column and type
/// settings it inherited from the real viewport.
fn place_clone(clone: &HtmlElement, left: f64, width: f64) {
    let style = clone.style();
    let _ = style.set_property("left", &format!("{left:.2}px"));
    let _ = style.set_property("width", &format!("{width:.2}px"));
}

fn clear_children(el: &HtmlElement) {
    el.set_inner_html("");
}

/// One face of a strip: a clipped window onto a column clone, or blank
/// paper for the back of a single sheet.
fn make_face(
    doc: &web_sys::Document,
    content: Option<(&Element, f64)>,
    stride: f64,
    rear: bool,
) -> Option<(Element, HtmlElement)> {
    let face = doc.create_element("div").ok()?;
    face.set_class_name(if rear { "flip-face rear" } else { "flip-face" });
    match content {
        Some((template, clip_left)) => {
            let clone = template.clone_node_with_deep(true).ok()?;
            let clone: HtmlElement = clone.dyn_into().ok()?;
            place_clone(&clone, -clip_left, stride);
            face.append_child(&clone).ok()?;
        }
        None => face.class_list().add_1("blank").ok()?,
    }
    let tint = doc.create_element("div").ok()?;
    tint.set_class_name("flip-tint");
    face.append_child(&tint).ok()?;
    Some((face, tint.dyn_into().ok()?))
}

/// A clone of the viewport whose flow reproduces exactly `column`: a spacer
/// consumes the columns before it (and, for a paragraph split across the
/// break, the part that sits in the previous column), then the elements
/// that intersect the column follow. Line breaks depend only on width, so
/// the clone paginates like the original at a fraction of the cost.
fn column_template(viewport: &HtmlElement, column: usize, geo: &Geometry) -> Option<Element> {
    let body = viewport.query_selector(".chapter-body").ok()??;
    let doc = document();
    let vp_rect = viewport.get_bounding_client_rect();
    let origin_x = vp_rect.left() - f64::from(viewport.scroll_left()) + geo.pad.left;
    let origin_y = vp_rect.top() + geo.pad.top;
    let pitch = geo.pitch();
    let column_height = geo.column_height();
    let target = column as isize;

    let children = body.children();
    let count = children.length();
    let mut first: Option<(u32, f64)> = None;
    let mut last: Option<u32> = None;
    for i in 0..count {
        let el = children.item(i)?;
        let rects = el.get_client_rects();
        let mut fragments: Vec<(isize, f64)> = Vec::new();
        for r in 0..rects.length() {
            let Some(rect) = rects.item(r) else { continue };
            if rect.width() <= 0.0 && rect.height() <= 0.0 {
                continue;
            }
            let col = ((rect.left() - origin_x) / pitch).round() as isize;
            fragments.push((col, rect.top() - origin_y));
        }
        let Some(min_col) = fragments.iter().map(|f| f.0).min() else {
            continue;
        };
        let max_col = fragments.iter().map(|f| f.0).max().unwrap_or(min_col);
        if max_col < target {
            continue;
        }
        if min_col > target {
            break;
        }
        if first.is_none() {
            let (c0, t0) = fragments
                .iter()
                .copied()
                .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))?;
            let spacer = c0.max(0) as f64 * column_height + t0.max(0.0);
            first = Some((i, spacer));
        }
        last = Some(i);
    }
    let (first_index, spacer) = first?;
    let last_index = last.unwrap_or(first_index);

    let template: Element = viewport.clone_node_with_deep(false).ok()?.dyn_into().ok()?;
    template.class_list().add_1("clone").ok()?;
    let body_clone: Element = body.clone_node_with_deep(false).ok()?.dyn_into().ok()?;
    if first_index > 0 {
        body_clone.class_list().add_1("clone").ok()?;
    }
    if spacer > 0.0 {
        let gap = doc.create_element("div").ok()?;
        set_style(&gap, &format!("height:{spacer:.2}px"));
        body_clone.append_child(&gap).ok()?;
    }
    let end = (last_index + 1).min(count.saturating_sub(1));
    for i in first_index..=end {
        let el = children.item(i)?;
        body_clone
            .append_child(&el.clone_node_with_deep(true).ok()?)
            .ok()?;
    }
    template.append_child(&body_clone).ok()?;
    Some(template)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn travel_maps_to_the_full_half_turn() {
        assert!(angle_for_travel(0.0, 400.0).abs() < 1e-9);
        assert!((angle_for_travel(200.0, 400.0) - 90.0).abs() < 1e-9);
        assert!((angle_for_travel(400.0, 400.0) - 180.0).abs() < 1e-9);
        assert!((angle_for_travel(900.0, 400.0) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn page_x_follows_the_column_layout() {
        let spread = Geometry {
            stride: 1000.0,
            height: 600.0,
            mode: Mode::Spread,
            pad: Paddings::default(),
        };
        assert_eq!(spread.page_x(0), 0.0);
        assert_eq!(spread.page_x(1), 500.0);
        assert_eq!(spread.page_x(2), 1000.0);
        assert_eq!(spread.page_x(3), 1500.0);
        let single = Geometry {
            mode: Mode::Single,
            ..spread
        };
        assert_eq!(single.page_x(3), 3000.0);
    }
}
