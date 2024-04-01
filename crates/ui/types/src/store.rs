use std::slice::Iter;

use render_api::base::Color;

use crate::{
    panel::{Panel, PanelStyle},
    style::{NodeStyle, StyleId, WidgetStyle},
    widget::WidgetKind,
    Button, ButtonStyle, NodeId, UiNode, TextStyle, Text,
    TextboxStyle, Textbox
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

    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

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

    pub(crate) fn get_style(&self, style_id: &StyleId) -> Option<&NodeStyle> {
        self.styles.get(style_id.as_usize())
    }

    pub(crate) fn get_style_mut(&mut self, style_id: &StyleId) -> Option<&mut NodeStyle> {
        self.styles.get_mut(style_id.as_usize())
    }

    // refs stuff

    pub fn node_kind(&self, node_id: &NodeId) -> WidgetKind {
        self.get_node(node_id).unwrap().widget_kind()
    }

    pub fn node_background_color(&self, node_id: &NodeId) -> Option<&Color> {
        match self.widget_style(node_id)? {
            WidgetStyle::Text(text_style) => text_style.background_color.as_ref(),
            WidgetStyle::Button(button_style) => button_style.panel.background_color.as_ref(),
            WidgetStyle::Textbox(textbox_style) => textbox_style.background_color.as_ref(),
            WidgetStyle::Panel(panel_style) => panel_style.background_color.as_ref(),
        }
    }

    pub fn node_style(&self, node_id: &NodeId) -> Option<&NodeStyle> {
        let node = self.get_node(node_id)?;
        node.style_id().map(|style_id| self.get_style(&style_id)).flatten()
    }

    fn widget_style(&self, node_id: &NodeId) -> Option<&WidgetStyle> {
        let style = self.node_style(node_id)?;
        Some(&style.base.widget_style)
    }

    pub fn panel_style(&self, node_id: &NodeId) -> Option<&PanelStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Panel(panel_style) => Some(panel_style),
            WidgetStyle::Button(button_style) => Some(&button_style.panel),
            _ => None,
        }
    }

    pub fn text_style(&self, node_id: &NodeId) -> Option<&TextStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Text(text_style) => Some(text_style),
            _ => None,
        }
    }

    pub fn button_style(&self, node_id: &NodeId) -> Option<&ButtonStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Button(button_style) => Some(button_style),
            _ => None,
        }
    }

    pub fn textbox_style(&self, node_id: &NodeId) -> Option<&TextboxStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Textbox(textbox_style) => Some(textbox_style),
            _ => None,
        }
    }

    // node widget-specific
    pub fn panel_ref(&self, node_id: &NodeId) -> Option<&Panel> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Panel {
            return node.widget_panel_ref();
        }
        None
    }

    pub fn text_ref(&self, node_id: &NodeId) -> Option<&Text> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Text {
            return node.widget_text_ref();
        }
        None
    }

    pub fn button_ref(&self, node_id: &NodeId) -> Option<&Button> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Button {
            return node.widget_button_ref();
        }
        None
    }

    pub fn button_mut(&mut self, node_id: &NodeId) -> Option<&mut Button> {
        let node = self.get_node_mut(node_id)?;
        if node.widget_kind() == WidgetKind::Button {
            return node.widget_button_mut();
        }
        None
    }

    pub fn textbox_ref(&self, node_id: &NodeId) -> Option<&Textbox> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Textbox {
            return node.widget_textbox_ref();
        }
        None
    }

    pub fn textbox_mut(&mut self, node_id: &NodeId) -> Option<&mut Textbox> {
        let node = self.get_node_mut(node_id)?;
        if node.widget_kind() == WidgetKind::Textbox {
            return node.widget_textbox_mut();
        }
        None
    }
}
