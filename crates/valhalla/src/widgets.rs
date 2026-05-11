//! Widget renderers.
//!
//! Each function takes the Rust mirror of a node's props and returns an
//! `AnyElement`. The render walk in `render.rs` calls into here based on the
//! `tag` string emitted by JS. Most widgets are gpui-component primitives
//! wrapped to accept Tailwind classes / `style` props and route their
//! events back into JS via the handler-id table.

use std::sync::Arc;

use gpui::{
    div, prelude::*, px, AnyElement, App, ElementId, MouseButton, SharedString, Window,
};
use gpui_component::{
    button::{Button as GpuiButton, ButtonVariants},
    checkbox::Checkbox as GpuiCheckbox,
    divider::Divider as GpuiDivider,
    switch::Switch as GpuiSwitch,
    Disableable, Icon as GpuiIcon, Sizable, Size,
};

use crate::events;
use crate::icons::parse_icon_name;
use crate::runtime::JsHost;
use crate::scene::{ElementProps, NodeId};
use crate::style;
use crate::tailwind;

// ─── helpers ─────────────────────────────────────────────────────────────

pub(crate) fn element_id(node_id: NodeId, prefix: &str) -> ElementId {
    ElementId::Name(format!("{}-{}", prefix, node_id).into())
}

fn dispatch_async(js: &Arc<JsHost>, hid: u32, payload: serde_json::Value) {
    if let Err(err) = events::dispatch(js, hid, payload) {
        log::warn!("[valhalla] dispatch error: {}", err);
    }
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
    js: &Arc<JsHost>,
) -> AnyElement {
    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);

    if let Some(&hid) = props.handlers.get("onClick") {
        let js = js.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_event, _window, _cx| {
            dispatch_async(&js, hid, serde_json::json!({}));
        });
    }

    el.children(children).into_any_element()
}

pub fn render_pressable(
    node_id: NodeId,
    props: &ElementProps,
    children: Vec<AnyElement>,
    js: &Arc<JsHost>,
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
            let js = js.clone();
            el = el.on_mouse_down(MouseButton::Left, move |event, _window, _cx| {
                let pos = event.position;
                dispatch_async(
                    &js,
                    hid,
                    serde_json::json!({
                        "x": f32::from(pos.x),
                        "y": f32::from(pos.y),
                    }),
                );
            });
        }
    }

    el.children(children).into_any_element()
}

// ─── text ────────────────────────────────────────────────────────────────

pub fn render_text(
    props: &ElementProps,
    children: Vec<AnyElement>,
) -> AnyElement {
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
    let mut div = if vertical {
        GpuiDivider::vertical()
    } else {
        GpuiDivider::horizontal()
    };
    if let Some(label) = label {
        div = div.label(SharedString::from(label.to_string()));
    }
    div.into_any_element()
}

// ─── badge ───────────────────────────────────────────────────────────────

pub fn render_badge(
    props: &ElementProps,
    children: Vec<AnyElement>,
) -> AnyElement {
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
    js: &Arc<JsHost>,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    let label = attr_str(props, "label").map(|s| s.to_string());
    let icon = attr_str(props, "icon").and_then(parse_icon_name);
    let variant = attr_str(props, "variant").unwrap_or("primary");
    let size = attr_str(props, "size").unwrap_or("md");
    let disabled = attr_bool(props, "disabled");

    let mut btn = GpuiButton::new(element_id(node_id, "btn"));
    if let Some(label) = label {
        btn = btn.label(SharedString::from(label));
    }
    if let Some(icon) = icon {
        btn = btn.icon(icon);
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
        let js = js.clone();
        btn = btn.on_click(move |_event, _window, _cx| {
            dispatch_async(&js, hid, serde_json::json!({}));
        });
    }

    btn.into_any_element()
}

// ─── icon ────────────────────────────────────────────────────────────────

pub fn render_icon(props: &ElementProps) -> AnyElement {
    let name = match attr_str(props, "name").and_then(parse_icon_name) {
        Some(n) => n,
        None => {
            // Unknown / missing — render an empty box so the layout doesn't
            // collapse and the typo is visually obvious.
            return GpuiIcon::empty().into_any_element();
        }
    };
    let size = attr_str(props, "size").unwrap_or("md");

    let mut icon = GpuiIcon::new(name);
    icon = match size {
        "xs" => icon.with_size(Size::XSmall),
        "sm" => icon.with_size(Size::Small),
        "lg" => icon.with_size(Size::Large),
        _ => icon.with_size(Size::Medium),
    };
    // Classes / style still apply via gpui_component::Icon's Styled impl,
    // but our tailwind/style appliers only target `Div`. Plain class-based
    // colouring on an Icon is V2; for now `Icon` ignores className/style.
    let _ = (&props.classes, &props.style);
    icon.into_any_element()
}

// ─── checkbox (gpui-component) ───────────────────────────────────────────

pub fn render_checkbox(
    node_id: NodeId,
    props: &ElementProps,
    js: &Arc<JsHost>,
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
        let js = js.clone();
        cb = cb.on_click(move |new_value: &bool, _window, _cx| {
            dispatch_async(&js, hid, serde_json::json!({ "value": *new_value }));
        });
    }
    cb.into_any_element()
}

// ─── switch (gpui-component) ─────────────────────────────────────────────

pub fn render_switch(
    node_id: NodeId,
    props: &ElementProps,
    js: &Arc<JsHost>,
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
        let js = js.clone();
        sw = sw.on_click(move |new_value: &bool, _window, _cx| {
            dispatch_async(&js, hid, serde_json::json!({ "value": *new_value }));
        });
    }
    sw.into_any_element()
}
