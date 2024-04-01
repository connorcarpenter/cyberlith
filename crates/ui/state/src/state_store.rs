use ui_runtime_config::{UiId, UiNodeR};
use crate::{panel::PanelState, button::ButtonState, UiNodeState, text::TextState, textbox::TextboxState};

pub struct UiStateStore {
    pub nodes: Vec<UiNodeState>,
}

impl UiStateStore {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    // nodes
    pub(crate) fn node_state_init(&mut self, node: &UiNodeR) {
        let node_state = UiNodeState::from_node(node);
        self.insert_node(node_state);
    }

    fn insert_node(&mut self, node: UiNodeState) {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        self.nodes.push(node);
    }

    pub fn get_node(&self, node_id: &UiId) -> Option<&UiNodeState> {
        self.nodes.get(node_id.as_usize())
    }

    pub(crate) fn get_node_mut(&mut self, node_id: &UiId) -> Option<&mut UiNodeState> {
        self.nodes.get_mut(node_id.as_usize())
    }

    pub(crate) fn node_ids(&self) -> Vec<UiId> {
        let mut output = Vec::new();

        for i in 0..self.nodes.len() {
            output.push(UiId::new(i as u32));
        }

        output
    }

    // refs stuff

    pub fn panel_ref(&self, node_id: &UiId) -> Option<&PanelState> {
        self.get_node(node_id)?.widget_panel_ref()
    }

    pub fn text_ref(&self, node_id: &UiId) -> Option<&TextState> {
        self.get_node(node_id)?.widget_text_ref()
    }

    pub fn button_ref(&self, node_id: &UiId) -> Option<&ButtonState> {
        self.get_node(node_id)?.widget_button_ref()
    }

    pub fn button_mut(&mut self, node_id: &UiId) -> Option<&mut ButtonState> {
        self.get_node_mut(node_id)?.widget_button_mut()
    }

    pub fn textbox_ref(&self, node_id: &UiId) -> Option<&TextboxState> {
        self.get_node(node_id)?.widget_textbox_ref()
    }

    pub fn textbox_mut(&mut self, node_id: &UiId) -> Option<&mut TextboxState> {
        self.get_node_mut(node_id)?.widget_textbox_mut()
    }
}
