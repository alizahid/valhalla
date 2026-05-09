//! The mirror tree.
//!
//! `SceneTree` is the canonical, Rust-side representation of what JS told us
//! to render. JS owns no DOM — the React reconciler emits `Op` mutations,
//! they arrive at `__host_commit` as a JSON array, and `SceneTree::apply`
//! reduces them onto the tree. The render walk in `render.rs` then turns
//! the tree into GPUI elements every frame the entity is dirtied.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value as JsonValue;

pub type NodeId = u32;

/// The container's parent ID. Used by `appendChildToContainer` etc.
pub const ROOT_PARENT: NodeId = 0;

/// One mutation. Field set is intentionally loose so we can evolve the protocol
/// without a Rust release: the JS side only sends what changed.
#[derive(Debug, Deserialize)]
#[serde(tag = "op")]
pub enum Op {
    /// Create an element node.
    #[serde(rename = "Create")]
    Create { id: NodeId, tag: String },

    /// Create a text node (leaf, just a string value).
    #[serde(rename = "CreateText")]
    CreateText { id: NodeId, value: String },

    /// Replace the prop bundle on an existing element.
    /// Sent on `createInstance` and on every `commitUpdate`.
    #[serde(rename = "SetProps")]
    SetProps {
        id: NodeId,
        #[serde(default)]
        classes: Vec<String>,
        #[serde(default)]
        style: HashMap<String, JsonValue>,
        #[serde(default)]
        handlers: HashMap<String, u32>,
        #[serde(default)]
        attrs: HashMap<String, JsonValue>,
    },

    /// Replace a text node's content.
    #[serde(rename = "UpdateText")]
    UpdateText { id: NodeId, value: String },

    /// Append `child` as the last child of `parent`.
    /// `parent == ROOT_PARENT (0)` means "the container".
    #[serde(rename = "Append")]
    Append { parent: NodeId, child: NodeId },

    /// Insert `child` immediately before `before` under `parent`.
    #[serde(rename = "Insert")]
    Insert {
        parent: NodeId,
        child: NodeId,
        before: NodeId,
    },

    /// Detach `child` from `parent`. Detached children are kept in the table
    /// (React may re-attach later); we drop on `Drop`.
    #[serde(rename = "Remove")]
    Remove { parent: NodeId, child: NodeId },

    /// Drop a node and (recursively) its descendants from the table.
    #[serde(rename = "Drop")]
    Drop { id: NodeId },

    /// Set the root child of the container.
    #[serde(rename = "SetRoot")]
    SetRoot { id: NodeId },
}

#[derive(Debug, Default)]
pub struct ElementProps {
    pub classes: Vec<String>,
    pub style: HashMap<String, JsonValue>,
    pub handlers: HashMap<String, u32>,
    pub attrs: HashMap<String, JsonValue>,
}

#[derive(Debug)]
pub enum Node {
    Element {
        tag: String,
        props: ElementProps,
        children: Vec<NodeId>,
    },
    Text {
        value: String,
    },
}

#[derive(Debug, Default)]
pub struct SceneTree {
    nodes: HashMap<NodeId, Node>,
    root: Option<NodeId>,
}

impl SceneTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Apply a single mutation. Errors are best-effort logged; protocol is
    /// expected to be coherent because both sides ship together.
    pub fn apply(&mut self, op: Op) {
        match op {
            Op::Create { id, tag } => {
                self.nodes.insert(
                    id,
                    Node::Element {
                        tag,
                        props: ElementProps::default(),
                        children: Vec::new(),
                    },
                );
            }
            Op::CreateText { id, value } => {
                self.nodes.insert(id, Node::Text { value });
            }
            Op::SetProps {
                id,
                classes,
                style,
                handlers,
                attrs,
            } => {
                if let Some(Node::Element { props, .. }) = self.nodes.get_mut(&id) {
                    props.classes = classes;
                    props.style = style;
                    props.handlers = handlers;
                    props.attrs = attrs;
                }
            }
            Op::UpdateText { id, value } => {
                if let Some(Node::Text { value: v }) = self.nodes.get_mut(&id) {
                    *v = value;
                }
            }
            Op::Append { parent, child } => {
                if parent == ROOT_PARENT {
                    self.root = Some(child);
                    return;
                }
                if let Some(Node::Element { children, .. }) = self.nodes.get_mut(&parent) {
                    children.retain(|&c| c != child);
                    children.push(child);
                }
            }
            Op::Insert {
                parent,
                child,
                before,
            } => {
                if parent == ROOT_PARENT {
                    self.root = Some(child);
                    return;
                }
                if let Some(Node::Element { children, .. }) = self.nodes.get_mut(&parent) {
                    children.retain(|&c| c != child);
                    if let Some(idx) = children.iter().position(|&c| c == before) {
                        children.insert(idx, child);
                    } else {
                        children.push(child);
                    }
                }
            }
            Op::Remove { parent, child } => {
                if parent == ROOT_PARENT {
                    if self.root == Some(child) {
                        self.root = None;
                    }
                    return;
                }
                if let Some(Node::Element { children, .. }) = self.nodes.get_mut(&parent) {
                    children.retain(|&c| c != child);
                }
            }
            Op::Drop { id } => {
                self.drop_recursive(id);
            }
            Op::SetRoot { id } => {
                self.root = Some(id);
            }
        }
    }

    fn drop_recursive(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.remove(&id) {
            if let Node::Element { children, .. } = node {
                for child in children {
                    self.drop_recursive(child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(json: &str) -> Op {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn creates_and_appends_element() {
        let mut t = SceneTree::new();
        t.apply(op(r#"{"op":"Create","id":1,"tag":"div"}"#));
        t.apply(op(
            r#"{"op":"SetProps","id":1,"classes":["flex"],"style":{},"handlers":{},"attrs":{}}"#,
        ));
        t.apply(op(r#"{"op":"Append","parent":0,"child":1}"#));
        assert_eq!(t.root(), Some(1));
    }

    #[test]
    fn insert_before_orders_children() {
        let mut t = SceneTree::new();
        t.apply(op(r#"{"op":"Create","id":1,"tag":"div"}"#));
        t.apply(op(r#"{"op":"Create","id":2,"tag":"span"}"#));
        t.apply(op(r#"{"op":"Create","id":3,"tag":"span"}"#));
        t.apply(op(r#"{"op":"Append","parent":1,"child":2}"#));
        t.apply(op(r#"{"op":"Insert","parent":1,"child":3,"before":2}"#));
        if let Some(Node::Element { children, .. }) = t.get(1) {
            assert_eq!(children, &[3, 2]);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn drop_recursive_removes_subtree() {
        let mut t = SceneTree::new();
        t.apply(op(r#"{"op":"Create","id":1,"tag":"div"}"#));
        t.apply(op(r#"{"op":"Create","id":2,"tag":"span"}"#));
        t.apply(op(r#"{"op":"Append","parent":1,"child":2}"#));
        t.apply(op(r#"{"op":"Drop","id":1}"#));
        assert!(t.get(1).is_none());
        assert!(t.get(2).is_none());
    }
}
