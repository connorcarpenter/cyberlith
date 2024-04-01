use ui_runner_config::{NodeId, WidgetKind};

use crate::{
    button::ButtonState, panel::PanelState, text::TextState, textbox::TextboxState, UiNodeState,
};

pub struct UiStateStore {
    pub nodes: Vec<UiNodeState>,
}

impl UiStateStore {
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    // nodes
    pub(crate) fn node_state_init(&mut self, widget_kind: &WidgetKind) {
        let node_state = UiNodeState::from_node(widget_kind);
        self.insert_node(node_state);
    }

    fn insert_node(&mut self, node: UiNodeState) {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        self.nodes.push(node);
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&UiNodeState> {
        self.nodes.get(id.as_usize())
    }

    pub(crate) fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut UiNodeState> {
        self.nodes.get_mut(id.as_usize())
    }

    pub(crate) fn node_ids(&self) -> Vec<NodeId> {
        let mut output = Vec::new();

        for i in 0..self.nodes.len() {
            output.push(NodeId::new(i as u32));
        }

        output
    }

    // refs stuff

    pub fn panel_ref(&self, id: &NodeId) -> Option<&PanelState> {
        self.get_node(id)?.widget_panel_ref()
    }

    pub fn text_ref(&self, id: &NodeId) -> Option<&TextState> {
        self.get_node(id)?.widget_text_ref()
    }

    pub fn button_ref(&self, id: &NodeId) -> Option<&ButtonState> {
        self.get_node(id)?.widget_button_ref()
    }

    pub fn button_mut(&mut self, id: &NodeId) -> Option<&mut ButtonState> {
        self.get_node_mut(id)?.widget_button_mut()
    }

    pub fn textbox_ref(&self, id: &NodeId) -> Option<&TextboxState> {
        self.get_node(id)?.widget_textbox_ref()
    }

    pub fn textbox_mut(&mut self, id: &NodeId) -> Option<&mut TextboxState> {
        self.get_node_mut(id)?.widget_textbox_mut()
    }
}
