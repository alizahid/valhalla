//! Walk `SceneTree` and produce GPUI elements.

use gpui::{div, prelude::*, AnyElement, MouseButton, SharedString};

use crate::events;
use crate::runtime::RootView;
use crate::scene::{ElementProps, Node, NodeId, SceneTree};
use crate::style;
use crate::tailwind;

pub fn render_root(this: &mut RootView) -> impl IntoElement {
    let root_id = this.scene.root();
    let mut container = div().size_full();

    if let Some(id) = root_id {
        let element = render_node(&this.scene, id, this);
        container = container.child(element);
    } else {
        container = container.child(div().child(SharedString::from(
            "[valhalla] no root rendered yet — bundle is loading.",
        )));
    }
    container
}

fn render_node(scene: &SceneTree, id: NodeId, view: &RootView) -> AnyElement {
    match scene.get(id) {
        Some(Node::Text { value }) => SharedString::from(value.clone()).into_any_element(),
        Some(Node::Element {
            tag,
            props,
            children,
        }) => {
            if tag == "input" {
                return crate::input::render_input(id, props, view);
            }
            render_div(scene, props, children, view)
        }
        None => div().into_any_element(),
    }
}

fn render_div(
    scene: &SceneTree,
    props: &ElementProps,
    children: &[NodeId],
    view: &RootView,
) -> AnyElement {
    let mut el = div();
    for op in tailwind::parse(&props.classes) {
        el = op(el);
    }
    el = style::apply(el, &props.style);

    if let Some(&hid) = props.handlers.get("onClick") {
        let js = view.js.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_event, _window, _cx| {
            if let Err(err) = events::dispatch(&js, hid, serde_json::json!({})) {
                log::warn!("[valhalla] dispatch error: {}", err);
            }
        });
    }

    for &child in children {
        el = el.child(render_node(scene, child, view));
    }

    el.into_any_element()
}
