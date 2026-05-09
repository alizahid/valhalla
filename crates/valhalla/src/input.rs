//! `<input>` rendering.
//!
//! V1 ships a minimal text-display surface: classes/style are applied to a
//! div, the current `value` attr is shown inside it, and `onClick` works
//! the way it does on any other div. Real text editing (cursor, IME, focus,
//! `onChange` per keystroke) is the next milestone — it lands as a
//! `gpui-component::TextField` integration in `render_input_textfield`
//! (below, behind a TODO) once we pin the gpui / gpui-component versions
//! against each other.
//!
//! Why a stub: the V1 demo flow validates the JS↔Rust render pipeline. Text
//! editing brings in focus / cursor / IME concerns that are best done after
//! we've confirmed the rest of the loop renders pixels correctly.

use std::sync::Arc;

use gpui::{div, prelude::*, AnyElement, MouseButton, SharedString};

use crate::events;
use crate::runtime::JsHost;
use crate::scene::{ElementProps, NodeId};
use crate::style;
use crate::tailwind;

pub fn render_input(id: NodeId, props: &ElementProps, js: &Arc<JsHost>) -> AnyElement {
    let _ = id;

    let value = props
        .attrs
        .get("value")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let placeholder = props
        .attrs
        .get("placeholder")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let display: SharedString = if value.is_empty() {
        placeholder.to_string().into()
    } else {
        value.to_string().into()
    };

    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);
    el = el.child(display);

    if let Some(&hid) = props.handlers.get("onClick") {
        let js = js.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_event, _window, _cx| {
            let _ = events::dispatch(&js, hid, serde_json::json!({}));
        });
    }

    el.into_any_element()
}
