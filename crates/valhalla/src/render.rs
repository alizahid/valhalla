//! Walk `SceneTree` and produce GPUI elements.
//!
//! Each element node has a `tag` string set by the JS-side React component
//! wrapper. We dispatch on it here. Unknown tags fall through to a `view`
//! renderer so future primitives can be added in JS first without breaking
//! the host.

use gpui::{div, prelude::*, AnyElement, Context, SharedString, Window};
use gpui_component::ActiveTheme;

use crate::runtime::RootView;
use crate::scene::{ElementProps, Node, NodeId, SceneTree};
use crate::widgets;

pub fn render_root(
    this: &mut RootView,
    window: &mut Window,
    cx: &mut Context<RootView>,
) -> AnyElement {
    // gpui's text rendering uses `rem` units; rems multiply against
    // window.rem_size(). gpui-component's theme is the source of truth.
    // Apply font_family + default text_color from the theme too so any
    // text not styled by user classes is still legible (theme-correct).
    let theme = cx.theme();
    window.set_rem_size(theme.font_size);

    let mut container = div()
        .size_full()
        .font_family(theme.font_family.clone())
        .text_color(theme.foreground);

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
    // Resolve children up-front so each widget renderer receives a Vec it
    // can drop into `.children(...)`. This pulls each child through the
    // walk recursively.
    let rendered_children: Vec<AnyElement> = children
        .iter()
        .map(|&id| render_node(scene, id, window, cx))
        .collect();

    match tag {
        // Layout / structural primitives.
        "view" | "div" => widgets::render_view(props, rendered_children, cx),
        "pressable" => widgets::render_pressable(node_id, props, rendered_children, cx),
        "text" | "span" => widgets::render_text(props, rendered_children),
        "scrollview" => widgets::render_scrollview(node_id, props, rendered_children),

        // gpui-component widgets.
        "button" => widgets::render_button(node_id, props, cx),
        "checkbox" => widgets::render_checkbox(node_id, props, cx),
        "switch" => widgets::render_switch(node_id, props, cx),
        "divider" => widgets::render_divider(props),
        "badge" => widgets::render_badge(props, rendered_children),
        "svg" => widgets::render_svg(props),
        "image" => widgets::render_image(props),

        // Text input — still stubbed (renders the value as static text).
        "textinput" | "input" => crate::input::render_input(node_id, props, cx),

        // Unknown tag: render as a view so a typo or future primitive doesn't
        // make the whole window go blank.
        _ => widgets::render_view(props, rendered_children, cx),
    }
}
