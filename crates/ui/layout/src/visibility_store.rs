use crate::NodeId;

pub struct UiVisibilityStore {
    pub nodes: Vec<bool>,
}

impl UiVisibilityStore {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    // nodes
    pub fn node_state_init(&mut self) {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        self.nodes.push(true); // all nodes are initialized with full visibility
    }

    pub fn get_node_visibility(&self, id: &NodeId) -> Option<bool> {
        self.nodes.get(id.as_usize()).copied()
    }

    pub fn set_node_visibility(&mut self, id: &NodeId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(id.as_usize()) {
            *node = visible;
        }
    }
}