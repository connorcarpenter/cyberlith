use std::slice::Iter;

use render_api::base::Color;
use ui_builder_config::{BaseNodeStyle, ButtonStyle, PanelStyle, StyleId, TextboxStyle, TextStyle, UiStore, WidgetStyle};

use crate::{UiId, node::UiNodeR};

pub struct UiRuntimeStore {
    styles: Vec<BaseNodeStyle>,
    nodes: Vec<UiNodeR>,
}

impl UiRuntimeStore {
    pub fn new(store: UiStore) -> Self {

        let (styles, nodes) = store.decompose();

        let styles = styles.into_iter().map(|style| style.base).collect();
        let nodes = nodes.into_iter().map(|node| node.into()).collect();

        Self {
            styles,
            nodes,
        }
    }

    // nodes

    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes_iter(&self) -> Iter<'_, UiNodeR> {
        self.nodes.iter()
    }

    pub fn get_node(&self, node_id: &UiId) -> Option<&UiNodeR> {
        self.nodes.get(node_id.as_usize())
    }

    // styles

    pub fn get_style(&self, style_id: &StyleId) -> Option<&BaseNodeStyle> {
        self.styles.get(style_id.as_usize())
    }

    // refs stuff

    pub fn node_background_color(&self, node_id: &UiId) -> Option<&Color> {
        match self.widget_style(node_id)? {
            WidgetStyle::Text(text_style) => text_style.background_color.as_ref(),
            WidgetStyle::Button(button_style) => button_style.panel.background_color.as_ref(),
            WidgetStyle::Textbox(textbox_style) => textbox_style.background_color.as_ref(),
            WidgetStyle::Panel(panel_style) => panel_style.background_color.as_ref(),
        }
    }

    pub fn node_style(&self, node_id: &UiId) -> Option<&BaseNodeStyle> {
        let node = self.get_node(node_id)?;
        node.style_id().map(|style_id| self.get_style(&style_id)).flatten()
    }

    fn widget_style(&self, node_id: &UiId) -> Option<&WidgetStyle> {
        let style = self.node_style(node_id)?;
        Some(&style.widget_style)
    }

    pub fn panel_style(&self, node_id: &UiId) -> Option<&PanelStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Panel(panel_style) => Some(panel_style),
            WidgetStyle::Button(button_style) => Some(&button_style.panel),
            _ => None,
        }
    }

    pub fn text_style(&self, node_id: &UiId) -> Option<&TextStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Text(text_style) => Some(text_style),
            _ => None,
        }
    }

    pub fn button_style(&self, node_id: &UiId) -> Option<&ButtonStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Button(button_style) => Some(button_style),
            _ => None,
        }
    }

    pub fn textbox_style(&self, node_id: &UiId) -> Option<&TextboxStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Textbox(textbox_style) => Some(textbox_style),
            _ => None,
        }
    }
}
