use std::collections::HashMap;
use logging::warn;

use crate::NodeId;

pub struct UiVisibilityStore {
    pub nodes: HashMap<NodeId, bool>,
}

impl UiVisibilityStore {
    pub fn new() -> Self {
        Self { nodes: HashMap::new() }
    }

    // nodes
    pub fn add_node(&mut self, id: &NodeId, node_init_visible: bool) {
        if self.nodes.len() >= 255 {
            warn!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        self.nodes.insert(*id, node_init_visible);
    }

    pub fn delete_node(&mut self, id: &NodeId) {
        self.nodes.remove(id);
    }

    pub fn get_node_visibility(&self, id: &NodeId) -> Option<bool> {
        self.nodes.get(id).copied()
    }

    pub fn set_node_visibility(&mut self, id: &NodeId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            *node = visible;
        } else {
            panic!("node not found for id: {:?}", id);
        }
    }
}
