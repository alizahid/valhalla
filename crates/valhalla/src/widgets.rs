//! Widget renderers.
//!
//! Each function takes the Rust mirror of a node's props and returns an
//! `AnyElement`. The render walk in `render.rs` calls into here based on the
//! `tag` string emitted by JS. Most widgets are gpui-component primitives
//! wrapped to accept Tailwind classes / `style` props and route their
//! events back into JS via the handler-id table.
//!
//! Every event handler goes through `cx.listener(...)`, which captures the
//! root entity weakly. Inside the listener we (1) call into JS to fire the
//! React handler — which runs setState synchronously and pushes new ops to
//! `bridge.inbox` via `__host_commit` — and then (2) call `this.drain(cx)`
//! to apply those ops and trigger `cx.notify()` so GPUI re-renders. Without
//! the drain, clicks silently disappear after the first render.

use gpui::{
    div, img, prelude::*, px, AnyElement, ClickEvent, Context, ElementId, MouseButton, ObjectFit,
    SharedString,
};
use gpui_component::{
    button::{Button as GpuiButton, ButtonVariants},
    checkbox::Checkbox as GpuiCheckbox,
    separator::Separator as GpuiSeparator,
    switch::Switch as GpuiSwitch,
    Disableable, Icon as GpuiIcon, Sizable, Size,
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

// ─── view / pressable ────────────────────────────────────────────────────

pub fn render_view(
    props: &ElementProps,
    children: Vec<AnyElement>,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);

    if let Some(&hid) = props.handlers.get("onClick") {
        el = el.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _event, _window, cx| {
                dispatch_and_drain(this, hid, serde_json::json!({}), cx);
            }),
        );
    }

    el.children(children).into_any_element()
}

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
            el = el.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                    let pos = event.position;
                    let payload = serde_json::json!({
                        "x": f32::from(pos.x),
                        "y": f32::from(pos.y),
                    });
                    dispatch_and_drain(this, hid, payload, cx);
                }),
            );
        }
    }

    el.children(children).into_any_element()
}

// ─── text ────────────────────────────────────────────────────────────────

pub fn render_text(props: &ElementProps, children: Vec<AnyElement>) -> AnyElement {
    // Text is rendered as a flex-row container so multiple inline children
    // (interpolated strings, formatted spans) lay out correctly. The classes
    // / style on `<Text>` apply to the whole line.
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

// ─── divider ─────────────────────────────────────────────────────────────

pub fn render_divider(props: &ElementProps) -> AnyElement {
    let vertical = attr_bool(props, "vertical");
    let label = attr_str(props, "label");
    let mut sep = if vertical {
        GpuiSeparator::vertical()
    } else {
        GpuiSeparator::horizontal()
    };
    if let Some(label) = label {
        sep = sep.label(SharedString::from(label.to_string()));
    }
    sep.into_any_element()
}

// ─── badge ───────────────────────────────────────────────────────────────

pub fn render_badge(props: &ElementProps, children: Vec<AnyElement>) -> AnyElement {
    let variant = attr_str(props, "variant").unwrap_or("default");
    let (bg, fg) = match variant {
        "success" => (gpui::rgb(0xDCFCE7), gpui::rgb(0x166534)),
        "warning" => (gpui::rgb(0xFEF3C7), gpui::rgb(0x854D0E)),
        "danger" => (gpui::rgb(0xFEE2E2), gpui::rgb(0x991B1B)),
        "info" => (gpui::rgb(0xDBEAFE), gpui::rgb(0x1E40AF)),
        _ => (gpui::rgb(0xE5E7EB), gpui::rgb(0x374151)),
    };
    let mut el = div()
        .flex()
        .flex_row()
        .items_center()
        .px(px(8.0))
        .py(px(2.0))
        .rounded(px(9999.0))
        .text_xs()
        .bg(bg)
        .text_color(fg);
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);
    el.children(children).into_any_element()
}

// ─── button (gpui-component) ─────────────────────────────────────────────

pub fn render_button(
    node_id: NodeId,
    props: &ElementProps,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let label = attr_str(props, "label").map(|s| s.to_string());
    let icon_src = attr_str(props, "icon");
    let variant = attr_str(props, "variant").unwrap_or("primary");
    let size = attr_str(props, "size").unwrap_or("md");
    let disabled = attr_bool(props, "disabled");

    let mut btn = GpuiButton::new(element_id(node_id, "btn"));
    if let Some(label) = label {
        btn = btn.label(SharedString::from(label));
    }
    if let Some(src) = icon_src {
        let path = assets::resolve(src);
        let path_str = path.to_string_lossy().into_owned();
        btn = btn.icon(GpuiIcon::default().path(SharedString::from(path_str)));
    }
    btn = match variant {
        "secondary" => btn.outline(),
        "ghost" => btn.ghost(),
        "outline" => btn.outline(),
        "danger" => btn.danger(),
        "link" => btn.link(),
        _ => btn.primary(),
    };
    btn = match size {
        "xs" => btn.with_size(Size::XSmall),
        "sm" => btn.with_size(Size::Small),
        "lg" => btn.with_size(Size::Large),
        _ => btn.with_size(Size::Medium),
    };
    if disabled {
        btn = btn.disabled(true);
    }

    if let Some(&hid) = props
        .handlers
        .get("onPress")
        .or_else(|| props.handlers.get("onClick"))
    {
        btn = btn.on_click(cx.listener(move |this, _ev: &ClickEvent, _window, cx| {
            dispatch_and_drain(this, hid, serde_json::json!({}), cx);
        }));
    }

    btn.into_any_element()
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

/// Standalone `<Svg>`. Rendered via gpui-component's `Icon` so it inherits
/// the cascaded text color (gpui's plain `svg()` defaults to black, which is
/// invisible on dark backgrounds — the most common cause of "icons disappear"
/// in earlier versions).
pub fn render_svg(props: &ElementProps) -> AnyElement {
    let Some(src) = attr_str(props, "src") else {
        return div().into_any_element();
    };
    let path = assets::resolve(src);
    let path_str = path.to_string_lossy().into_owned();
    let s = size_to_px(attr_str(props, "size"));

    let mut icon = GpuiIcon::default().path(SharedString::from(path_str)).size(s);
    if let Some(tint) = attr_str(props, "tint").and_then(style::parse_color_public) {
        icon = icon.text_color(tint);
    }
    icon.into_any_element()
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

// ─── checkbox (gpui-component) ───────────────────────────────────────────

pub fn render_checkbox(
    node_id: NodeId,
    props: &ElementProps,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let checked = attr_bool(props, "checked");
    let disabled = attr_bool(props, "disabled");
    let label = attr_str(props, "label").map(|s| s.to_string());

    let mut cb = GpuiCheckbox::new(element_id(node_id, "cb")).checked(checked);
    if let Some(label) = label {
        cb = cb.label(SharedString::from(label));
    }
    if disabled {
        cb = cb.disabled(true);
    }
    if let Some(&hid) = props.handlers.get("onValueChange") {
        cb = cb.on_click(cx.listener(move |this, new_value: &bool, _window, cx| {
            dispatch_and_drain(this, hid, serde_json::json!({ "value": *new_value }), cx);
        }));
    }
    cb.into_any_element()
}

// ─── switch (gpui-component) ─────────────────────────────────────────────

pub fn render_switch(
    node_id: NodeId,
    props: &ElementProps,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let checked = attr_bool(props, "checked");
    let disabled = attr_bool(props, "disabled");
    let label = attr_str(props, "label").map(|s| s.to_string());

    let mut sw = GpuiSwitch::new(element_id(node_id, "sw")).checked(checked);
    if let Some(label) = label {
        sw = sw.label(SharedString::from(label));
    }
    if disabled {
        sw = sw.disabled(true);
    }
    if let Some(&hid) = props.handlers.get("onValueChange") {
        sw = sw.on_click(cx.listener(move |this, new_value: &bool, _window, cx| {
            dispatch_and_drain(this, hid, serde_json::json!({ "value": *new_value }), cx);
        }));
    }
    sw.into_any_element()
}
