use ui_types::{NodeId, UiNode};

use crate::{textbox_input_state::TextboxInputState, node_input_state::UiNodeInputState};

pub struct UiInputStateStore {
    pub nodes: Vec<UiNodeInputState>,
}

impl UiInputStateStore {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    // nodes
    pub(crate) fn node_state_init(&mut self, node: &UiNode) {
        let node_state = UiNodeInputState::from_node(node);
        self.insert_node(node_state);
    }

    fn insert_node(&mut self, node: UiNodeInputState) -> NodeId {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        NodeId::new(index as u32)
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&UiNodeInputState> {
        self.nodes.get(node_id.as_usize())
    }

    pub(crate) fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut UiNodeInputState> {
        self.nodes.get_mut(node_id.as_usize())
    }

    // refs stuff

    pub fn textbox_ref(&self, node_id: &NodeId) -> Option<&TextboxInputState> {
        self.get_node(node_id)?.widget_textbox_ref()
    }

    pub fn textbox_mut(&mut self, node_id: &NodeId) -> Option<&mut TextboxInputState> {
        self.get_node_mut(node_id)?.widget_textbox_mut()
    }
}
