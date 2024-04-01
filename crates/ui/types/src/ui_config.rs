use std::collections::HashMap;

use asset_id::AssetId;
use render_api::base::Color;

use crate::{
    node::UiNode,
    node_id::NodeId,
    panel::Panel,
    store::UiStore,
    style::{NodeStyle, StyleId},
    widget::Widget,
};

pub struct UiConfig {
    pub globals: Globals,
    pub store: UiStore,

    id_str_to_node_id_map: HashMap<String, NodeId>,
}

impl UiConfig {
    pub const ROOT_NODE_ID: NodeId = NodeId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            globals: Globals::new(),
            store: UiStore::new(),

            id_str_to_node_id_map: HashMap::new(),
        };

        // Root Node
        let root_panel_id = me.create_node(Widget::Panel(Panel::new()));
        if root_panel_id != Self::ROOT_NODE_ID {
            panic!("root panel id is not 0");
        }

        me
    }

    pub fn decompose(self) -> (UiStore, Globals, HashMap<String, NodeId>) {
        (self.store, self.globals, self.id_str_to_node_id_map)
    }

    // events
    pub fn get_first_input(&self) -> Option<NodeId> {
        self.globals.first_input
    }

    pub fn set_first_input(&mut self, id: NodeId) {
        self.globals.first_input = Some(id);
    }

    // interface

    pub fn get_text_icon_asset_id(&self) -> AssetId {
        *self.globals.text_icon_asset_id_opt.as_ref().unwrap()
    }

    pub fn set_text_icon_asset_id(&mut self, text_icon_asset_id: &AssetId) -> &mut Self {
        self.globals.text_icon_asset_id_opt = Some(text_icon_asset_id.clone());
        self
    }

    pub fn get_text_color(&self) -> Color {
        self.globals.text_color
    }

    pub fn set_text_color(&mut self, text_color: Color) -> &mut Self {
        self.globals.set_text_color(text_color);
        self
    }

    pub fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_id_map.get(id_str).cloned()
    }

    pub fn create_node(&mut self, widget: Widget) -> NodeId {
        let mut id_str_opt = None;
        match &widget {
            Widget::Button(button) => {
                id_str_opt = Some(button.id_str.clone());
            }
            Widget::Textbox(textbox) => {
                id_str_opt = Some(textbox.id_str.clone());
            }
            _ => {}
        }

        let ui_node = UiNode::new(widget);
        let node_id = self.store.insert_node(ui_node);

        if let Some(id_str) = id_str_opt {
            self.id_str_to_node_id_map.insert(id_str, node_id);
        }

        node_id
    }

    pub fn node_ref(&self, id: &NodeId) -> Option<&UiNode> {
        self.store.get_node(&id)
    }

    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNode> {
        self.store.get_node_mut(&id)
    }

    pub fn style_mut(&mut self, id: &StyleId) -> Option<&mut NodeStyle> {
        self.store.get_style_mut(&id)
    }

    pub fn insert_style(&mut self, style: NodeStyle) -> StyleId {
        self.store.insert_style(style)
    }
}

pub struct Globals {
    text_icon_asset_id_opt: Option<AssetId>,
    text_color: Color,
    first_input: Option<NodeId>,
}

impl Globals {
    pub(crate) fn new() -> Self {
        Self {
            text_icon_asset_id_opt: None,
            text_color: Color::BLACK,
            first_input: None,
        }
    }

    pub fn get_first_input_node_id(&self) -> NodeId {
        self.first_input.unwrap_or(UiConfig::ROOT_NODE_ID)
    }

    pub fn get_text_color(&self) -> Color {
        self.text_color
    }

    pub fn set_text_color(&mut self, color: Color) {
        if color == self.text_color {
            return;
        }
        self.text_color = color;
    }
}