use std::slice::Iter;

use crate::{
    style::{NodeStyle, StyleId},
    NodeId, UiNode,
};

pub struct UiStore {
    styles: Vec<NodeStyle>,
    nodes: Vec<UiNode>,
}

impl UiStore {
    pub(crate) fn new() -> Self {
        Self {
            styles: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn decompose(self) -> (Vec<NodeStyle>, Vec<UiNode>) {
        (self.styles, self.nodes)
    }

    // nodes

    pub fn nodes_iter(&self) -> Iter<'_, UiNode> {
        self.nodes.iter()
    }

    pub(crate) fn insert_node(&mut self, node: UiNode) -> NodeId {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        NodeId::new(index as u32)
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&UiNode> {
        self.nodes.get(node_id.as_usize())
    }

    pub(crate) fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(node_id.as_usize())
    }

    // styles

    pub fn styles_iter(&self) -> Iter<'_, NodeStyle> {
        self.styles.iter()
    }

    pub(crate) fn insert_style(&mut self, style: NodeStyle) -> StyleId {
        if self.styles.len() >= 255 {
            panic!("1 UI can only hold up to 255 styles, too many styles!");
        }
        let index = self.styles.len();
        self.styles.push(style);
        StyleId::new(index as u32)
    }

    pub(crate) fn get_style_mut(&mut self, style_id: &StyleId) -> Option<&mut NodeStyle> {
        self.styles.get_mut(style_id.as_usize())
    }
}
