//! `<input>` rendering.
//!
//! V1 ships a minimal text-display surface: classes/style are applied to a
//! div, the current `value` attr is shown inside it, and `onClick` works
//! the way it does on any other div. Real text editing (cursor, IME, focus,
//! `onChange` per keystroke) is the next milestone — it lands as a
//! gpui-component `Input` integration once we wire focus management.

use gpui::{div, prelude::*, AnyElement, Context, MouseButton, SharedString};

use crate::events;
use crate::runtime::RootView;
use crate::scene::{ElementProps, NodeId};
use crate::style;
use crate::tailwind;

pub fn render_input(
    _id: NodeId,
    props: &ElementProps,
    cx: &mut Context<RootView>,
) -> AnyElement {
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
        el = el.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _event, _window, cx| {
                if let Err(err) = events::dispatch(&this.js, hid, serde_json::json!({})) {
                    log::warn!("[valhalla] dispatch error: {}", err);
                }
                this.drain(cx);
            }),
        );
    }

    el.into_any_element()
}
