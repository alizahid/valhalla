//! Walk `SceneTree` and produce GPUI elements.
//!
//! Each element node has a `tag` string set by the JS-side React component
//! wrapper. We dispatch on it here. Unknown tags fall through to a `view`
//! renderer so future primitives can be added in JS first without breaking
//! the host.

use std::sync::Arc;

use gpui::{div, prelude::*, AnyElement, App, SharedString, Window};

use crate::runtime::{JsHost, RootView};
use crate::scene::{ElementProps, Node, NodeId, SceneTree};
use crate::widgets;

pub fn render_root(this: &mut RootView, window: &mut Window, cx: &mut App) -> AnyElement {
    let mut container = div().size_full();
    if let Some(id) = this.scene.root() {
        let element = render_node(&this.scene, id, &this.js, window, cx);
        container = container.child(element);
    } else {
        container = container.child(SharedString::from(
            "[valhalla] no root rendered yet — bundle is loading.",
        ));
    }
    container.into_any_element()
}

fn render_node(
    scene: &SceneTree,
    id: NodeId,
    js: &Arc<JsHost>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    match scene.get(id) {
        Some(Node::Text { value }) => SharedString::from(value.clone()).into_any_element(),

        Some(Node::Element {
            tag,
            props,
            children,
        }) => render_element(scene, id, tag, props, children, js, window, cx),

        None => div().into_any_element(),
    }
}

fn render_element(
    scene: &SceneTree,
    node_id: NodeId,
    tag: &str,
    props: &ElementProps,
    children: &[NodeId],
    js: &Arc<JsHost>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    // Resolve children up-front so each widget renderer receives a Vec it
    // can drop into `.children(...)`. This pulls each child through the
    // walk recursively.
    let rendered_children: Vec<AnyElement> = children
        .iter()
        .map(|&id| render_node(scene, id, js, window, cx))
        .collect();

    match tag {
        // Layout / structural primitives.
        "view" | "div" => widgets::render_view(props, rendered_children, js),
        "pressable" => widgets::render_pressable(node_id, props, rendered_children, js),
        "text" | "span" => widgets::render_text(props, rendered_children),
        "scrollview" => widgets::render_scrollview(node_id, props, rendered_children),

        // gpui-component widgets.
        "button" => widgets::render_button(node_id, props, js, window, cx),
        "checkbox" => widgets::render_checkbox(node_id, props, js),
        "switch" => widgets::render_switch(node_id, props, js),
        "divider" => widgets::render_divider(props),
        "badge" => widgets::render_badge(props, rendered_children),

        // Text input — still stubbed (renders the value as static text).
        "textinput" | "input" => crate::input::render_input(node_id, props, js),

        // Unknown tag: render as a view so a typo or future primitive doesn't
        // make the whole window go blank.
        _ => widgets::render_view(props, rendered_children, js),
    }
}
