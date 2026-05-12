//! Widget renderers — gpui primitives only.
//!
//! The framework deliberately ships a small set: View, Text, Pressable,
//! ScrollView, Svg, Image, TextInput. Everything else (buttons with
//! variants, checkboxes, switches, badges, dividers, dropdowns…) lives
//! in userland, built from these primitives. See `examples/kanban` for
//! how a real app composes them.
//!
//! Every event handler goes through `cx.listener(...)` so the click
//! closure can call `this.drain(cx)` after dispatching into JS — that
//! applies the new ops React just committed and triggers a repaint.

use gpui::{
    div, img, prelude::*, px, svg, AnyElement, ClickEvent, Context, ElementId, ObjectFit,
    SharedString,
};

use crate::assets;
use crate::events;
use crate::runtime::RootView;
use crate::scene::{ElementProps, NodeId};
use crate::style;
use crate::tailwind;

// ─── helpers ─────────────────────────────────────────────────────────────

pub(crate) fn element_id(node_id: NodeId, prefix: &str) -> ElementId {
    ElementId::Name(format!("{}-{}", prefix, node_id).into())
}

/// Fire a JS event handler by id, then drain any ops the handler pushed.
/// The drain calls `cx.notify()` if anything came in, scheduling a repaint.
fn dispatch_and_drain(
    this: &mut RootView,
    hid: u32,
    payload: serde_json::Value,
    cx: &mut Context<RootView>,
) {
    if let Err(err) = events::dispatch(&this.js, hid, payload) {
        log::warn!("[valhalla] dispatch error: {}", err);
    }
    this.drain(cx);
}

fn attr_bool(props: &ElementProps, key: &str) -> bool {
    props.attrs.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn attr_str<'a>(props: &'a ElementProps, key: &str) -> Option<&'a str> {
    props.attrs.get(key).and_then(|v| v.as_str())
}

// ─── view ────────────────────────────────────────────────────────────────

pub fn render_view(
    node_id: NodeId,
    props: &ElementProps,
    children: Vec<AnyElement>,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);

    // A plain View doesn't need an id unless it has an onClick — gpui
    // requires stateful elements (with id) for click handling.
    if let Some(&hid) = props.handlers.get("onClick") {
        let id = element_id(node_id, "view");
        let stateful = el.id(id);
        return stateful
            .on_click(cx.listener(move |this, _ev: &ClickEvent, _window, cx| {
                dispatch_and_drain(this, hid, serde_json::json!({}), cx);
            }))
            .children(children)
            .into_any_element();
    }

    el.children(children).into_any_element()
}

// ─── pressable ───────────────────────────────────────────────────────────
// Same shape as a clickable div in gpui's examples: an id'd div with
// `.cursor_pointer().on_click(...)`. `onPress` is the RN-style name; we
// accept `onClick` too for ergonomics.

pub fn render_pressable(
    node_id: NodeId,
    props: &ElementProps,
    children: Vec<AnyElement>,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    let el = style::apply(el, &props.style);
    let mut el = el.id(element_id(node_id, "press")).cursor_pointer();

    if !attr_bool(props, "disabled") {
        if let Some(&hid) = props
            .handlers
            .get("onPress")
            .or_else(|| props.handlers.get("onClick"))
        {
            el = el.on_click(cx.listener(move |this, ev: &ClickEvent, _window, cx| {
                let pos = ev.position();
                let payload = serde_json::json!({
                    "x": f32::from(pos.x),
                    "y": f32::from(pos.y),
                });
                dispatch_and_drain(this, hid, payload, cx);
            }));
        }
    }

    el.children(children).into_any_element()
}

// ─── text ────────────────────────────────────────────────────────────────

pub fn render_text(props: &ElementProps, children: Vec<AnyElement>) -> AnyElement {
    // Text is a flex-row container so multiple inline children (interpolated
    // strings, formatted spans) lay out correctly. Classes / style on
    // `<Text>` apply to the whole line; descendants inherit text color +
    // font size via gpui's normal cascade.
    let mut el = div().flex_row();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);
    el.children(children).into_any_element()
}

// ─── scrollview ──────────────────────────────────────────────────────────

pub fn render_scrollview(
    node_id: NodeId,
    props: &ElementProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    let horizontal = attr_bool(props, "horizontal");
    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    let el = style::apply(el, &props.style);
    let el = el.id(element_id(node_id, "scroll"));
    let el = if horizontal {
        el.overflow_x_scroll()
    } else {
        el.overflow_y_scroll()
    };
    el.children(children).into_any_element()
}

// ─── svg ─────────────────────────────────────────────────────────────────

fn size_to_px(s: Option<&str>) -> gpui::Pixels {
    match s {
        Some("xs") => px(12.0),
        Some("sm") => px(16.0),
        Some("lg") => px(24.0),
        Some(other) => other.parse::<f32>().map(px).unwrap_or(px(20.0)),
        None => px(20.0),
    }
}

/// `gpui::svg()` for SVG paths. Tint is applied via `text_color()` —
/// gpui's SVG renderer uses that as the fill colour. Users who want
/// inheritance from a surrounding theme can do their own wrapper.
pub fn render_svg(props: &ElementProps) -> AnyElement {
    let Some(src) = attr_str(props, "src") else {
        return div().into_any_element();
    };
    let path = assets::resolve(src);
    let path_str = path.to_string_lossy().into_owned();
    let s = size_to_px(attr_str(props, "size"));

    let mut el = svg().path(SharedString::from(path_str)).w(s).h(s);
    if let Some(c) = attr_str(props, "tint").and_then(style::parse_color_public) {
        el = el.text_color(c);
    }
    el.into_any_element()
}

// ─── image ───────────────────────────────────────────────────────────────

pub fn render_image(props: &ElementProps) -> AnyElement {
    let Some(src) = attr_str(props, "src") else {
        return div().into_any_element();
    };
    let path = assets::resolve(src);
    let path_str = path.to_string_lossy().into_owned();

    let mut el = img(SharedString::from(path_str));
    if let Some(w) = props.attrs.get("width").and_then(|v| v.as_f64()) {
        el = el.w(px(w as f32));
    }
    if let Some(h) = props.attrs.get("height").and_then(|v| v.as_f64()) {
        el = el.h(px(h as f32));
    }
    if let Some(fit) = attr_str(props, "objectFit") {
        el = el.object_fit(match fit {
            "cover" => ObjectFit::Cover,
            "fill" => ObjectFit::Fill,
            "none" => ObjectFit::None,
            _ => ObjectFit::Contain,
        });
    }
    el.into_any_element()
}
