//! Walk `SceneTree` and produce GPUI elements.
//!
//! The framework ships only a handful of primitives — anything else
//! (buttons, badges, checkboxes, switches…) is built in userland from
//! these primitives. Unknown tags fall through to a plain View renderer
//! so userland React components that resolve to unknown host elements
//! (typos, future primitives) don't blank the window.

use gpui::{div, prelude::*, AnyElement, Context, SharedString, Window};

use crate::runtime::RootView;
use crate::scene::{ElementProps, Node, NodeId, SceneTree};
use crate::widgets;

pub fn render_root(
    this: &mut RootView,
    window: &mut Window,
    cx: &mut Context<RootView>,
) -> AnyElement {
    // gpui defaults to a 16px rem and the platform's UI font; that's fine
    // for our purposes — the consumer's React app drives styling via
    // Tailwind classes / style props from the tree on down.
    let _ = window;

    let mut container = div().size_full();

    if let Some(id) = this.scene.root() {
        let element = render_node(&this.scene, id, window, cx);
        container = container.child(element);
    } else {
        container = container.child(SharedString::from(
            "[valhalla] no root rendered yet — bundle is loading.",
        ));
    }
    container.into_any_element()
}

pub(crate) fn render_node(
    scene: &SceneTree,
    id: NodeId,
    window: &mut Window,
    cx: &mut Context<RootView>,
) -> AnyElement {
    match scene.get(id) {
        Some(Node::Text { value }) => SharedString::from(value.clone()).into_any_element(),

        Some(Node::Element {
            tag,
            props,
            children,
        }) => render_element(scene, id, tag, props, children, window, cx),

        None => div().into_any_element(),
    }
}

fn render_element(
    scene: &SceneTree,
    node_id: NodeId,
    tag: &str,
    props: &ElementProps,
    children: &[NodeId],
    window: &mut Window,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let rendered_children: Vec<AnyElement> = children
        .iter()
        .map(|&id| render_node(scene, id, window, cx))
        .collect();

    match tag {
        // Primitives — these are the entire framework surface.
        "view" | "div" => widgets::render_view(node_id, props, rendered_children, cx),
        "pressable" => widgets::render_pressable(node_id, props, rendered_children, cx),
        "text" | "span" => widgets::render_text(props, rendered_children),
        "scrollview" => widgets::render_scrollview(node_id, props, rendered_children),
        "svg" => widgets::render_svg(props),
        "image" | "img" => widgets::render_image(props),
        "textinput" | "input" => crate::input::render_input(node_id, props, cx),

        // Unknown tag: treat as a view so userland tags or typos don't
        // blank the window.
        _ => widgets::render_view(node_id, props, rendered_children, cx),
    }
}
