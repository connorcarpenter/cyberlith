use crate::NodeId;

pub struct UiVisibilityStore {
    pub nodes: Vec<bool>,
}

impl UiVisibilityStore {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    // nodes
    pub fn node_state_init(&mut self) {
        self.insert_node();
    }

    fn insert_node(&mut self) -> NodeId {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        let index = self.nodes.len();
        self.nodes.push(true); // all nodes are initialized with full visibility
        NodeId::new(index as u32)
    }

    pub fn get_node_visibility(&self, node_id: &NodeId) -> Option<bool> {
        self.nodes.get(node_id.as_usize()).copied()
    }

    pub fn set_node_visibility(&mut self, node_id: &NodeId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(node_id.as_usize()) {
            *node = visible;
        }
    }
}
