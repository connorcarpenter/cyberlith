use std::collections::HashMap;
use ui_runner_config::{NodeId, StyleId, WidgetKind};

use crate::{
    textbox::TextboxState, UiNodeState, style_state::StyleState,
};
use crate::button::ButtonStyleState;
use crate::panel::PanelStyleState;
use crate::text::TextStyleState;
use crate::textbox::TextboxStyleState;

pub struct UiStateStore {
    pub nodes: Vec<UiNodeState>,
    pub default_styles: HashMap<WidgetKind, StyleState>,
    pub styles: Vec<StyleState>,
}

impl UiStateStore {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            default_styles: HashMap::new(),
            styles: Vec::new()
        }
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

    pub fn textbox_ref(&self, id: &NodeId) -> Option<&TextboxState> {
        self.get_node(id)?.widget_textbox_ref()
    }

    pub fn textbox_mut(&mut self, id: &NodeId) -> Option<&mut TextboxState> {
        self.get_node_mut(id)?.widget_textbox_mut()
    }

    // styles

    pub(crate) fn get_style_state(&self, widget_kind: WidgetKind, style_id: Option<StyleId>) -> Option<&StyleState> {
        if let Some(style_id) = style_id {
            let style_id = style_id.as_usize();
            self.styles.get(style_id)
        } else {
            self.default_styles.get(&widget_kind)
        }
    }

    pub(crate) fn style_state_init(&mut self, widget_kind: &WidgetKind) {
        let style_state = StyleState::from_kind(widget_kind);
        self.insert_style(style_state);
    }

    fn insert_style(&mut self, style: StyleState) {
        if self.styles.len() >= 255 {
            panic!("1 UI can only hold up to 255 styles, too many styles!");
        }
        self.styles.push(style);
    }

    pub(crate) fn create_panel_style(&mut self, style_id: Option<StyleId>) -> Option<&mut PanelStyleState> {
        if let Some(style_id) = style_id {
            let style_id = style_id.as_usize();
            let Some(StyleState::Panel(style)) = self.styles.get_mut(style_id) else {
                panic!("Style not found");
            };
            if !style.needs_color_handle() {
                // style state already has color handles
                return None;
            }
            return Some(style);
        } else {
            if !self.default_styles.contains_key(&WidgetKind::Panel) {
                self.default_styles.insert(WidgetKind::Panel, StyleState::Panel(PanelStyleState::new()));
                let panel_style_state = self.default_styles.get_mut(&WidgetKind::Panel).unwrap();
                let StyleState::Panel(panel_style_state) = panel_style_state else {
                    panic!("impossible");
                };
                return Some(panel_style_state);
            } else {
                // default style state already initialized
                return None;
            }
        }
    }

    pub(crate) fn create_text_style(&mut self, style_id: Option<StyleId>) -> Option<&mut TextStyleState> {
        if let Some(style_id) = style_id {
            let style_id = style_id.as_usize();
            let Some(StyleState::Text(style)) = self.styles.get_mut(style_id) else {
                panic!("Style not found");
            };
            if !style.needs_color_handle() {
                // style state already has color handles
                return None;
            }
            return Some(style);
        } else {
            if !self.default_styles.contains_key(&WidgetKind::Text) {
                self.default_styles.insert(WidgetKind::Text, StyleState::Text(TextStyleState::new()));
                let text_style_state = self.default_styles.get_mut(&WidgetKind::Text).unwrap();
                let StyleState::Text(text_style_state) = text_style_state else {
                    panic!("impossible");
                };
                return Some(text_style_state);
            } else {
                // default style state already initialized
                return None;
            }
        }
    }

    pub(crate) fn create_button_style(&mut self, style_id: Option<StyleId>) -> Option<&mut ButtonStyleState> {
        if let Some(style_id) = style_id {
            let style_id = style_id.as_usize();
            let Some(StyleState::Button(style)) = self.styles.get_mut(style_id) else {
                panic!("Style not found");
            };
            if !style.needs_color_handle() {
                // style state already has color handles
                return None;
            }
            return Some(style);
        } else {
            if !self.default_styles.contains_key(&WidgetKind::Button) {
                self.default_styles.insert(WidgetKind::Button, StyleState::Button(ButtonStyleState::new()));
                let button_style_state = self.default_styles.get_mut(&WidgetKind::Button).unwrap();
                let StyleState::Button(button_style) = button_style_state else {
                    panic!("impossible");
                };
                return Some(button_style);
            } else {
                // default style state already initialized
                return None;
            }
        }
    }

    pub(crate) fn create_textbox_style(&mut self, style_id: Option<StyleId>) -> Option<&mut TextboxStyleState> {
        if let Some(style_id) = style_id {
            let style_id = style_id.as_usize();
            let Some(StyleState::Textbox(style)) = self.styles.get_mut(style_id) else {
                panic!("Style not found");
            };
            if !style.needs_color_handle() {
                // style state already has color handles
                return None;
            }
            return Some(style);
        } else {
            if !self.default_styles.contains_key(&WidgetKind::Textbox) {
                self.default_styles.insert(WidgetKind::Textbox, StyleState::Textbox(TextboxStyleState::new()));
                let textbox_style_state = self.default_styles.get_mut(&WidgetKind::Textbox).unwrap();
                let StyleState::Textbox(textbox_style_state) = textbox_style_state else {
                    panic!("impossible");
                };
                return Some(textbox_style_state);
            } else {
                // default style state already initialized
                return None;
            }
        }
    }
}