use std::collections::HashMap;
use std::slice::Iter;

use asset_id::AssetId;
use render_api::base::Color;
use ui_layout::NodeId;

use crate::{
    node::UiNode,
    panel::Panel,
    style::{NodeStyle, StyleId},
    widget::Widget,
};

pub struct UiConfig {
    styles: Vec<NodeStyle>,
    nodes: Vec<UiNode>,

    text_color: Color,
    first_input: Option<NodeId>,
    text_icon_asset_id_opt: Option<AssetId>,
    id_str_to_node_id_map: HashMap<String, NodeId>,
}

impl UiConfig {
    pub const ROOT_NODE_ID: NodeId = NodeId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            styles: Vec::new(),
            nodes: Vec::new(),

            text_color: Color::BLACK,
            first_input: None,
            text_icon_asset_id_opt: None,
            id_str_to_node_id_map: HashMap::new(),
        };

        // Root Node
        let root_panel_id = me.create_node(Widget::Panel(Panel::new()));
        if root_panel_id != Self::ROOT_NODE_ID {
            panic!("root panel id is not 0");
        }

        me
    }

    pub fn decompose(
        self,
    ) -> (
        Vec<NodeStyle>,
        Vec<UiNode>,
        Color,
        NodeId,
        AssetId,
        HashMap<String, NodeId>,
    ) {
        (
            self.styles,
            self.nodes,
            self.text_color,
            self.first_input.unwrap(),
            self.text_icon_asset_id_opt.unwrap(),
            self.id_str_to_node_id_map,
        )
    }

    // nodes

    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(id.as_usize())
    }

    pub fn nodes_iter(&self) -> Iter<'_, UiNode> {
        self.nodes.iter()
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
        let node_id = self.insert_node(ui_node);

        if let Some(id_str) = id_str_opt {
            self.id_str_to_node_id_map.insert(id_str, node_id);
        }

        node_id
    }

    fn insert_node(&mut self, node: UiNode) -> NodeId {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        NodeId::new(index as u32)
    }

    // styles

    pub fn style_mut(&mut self, id: &StyleId) -> Option<&mut NodeStyle> {
        self.styles.get_mut(id.as_usize())
    }

    pub fn styles_iter(&self) -> Iter<'_, NodeStyle> {
        self.styles.iter()
    }

    pub fn insert_style(&mut self, style: NodeStyle) -> StyleId {
        if self.styles.len() >= 255 {
            panic!("1 UI can only hold up to 255 styles, too many styles!");
        }
        let index = self.styles.len();
        self.styles.push(style);
        StyleId::new(index as u32)
    }

    // globals

    pub fn get_first_input(&self) -> Option<NodeId> {
        self.first_input
    }

    pub fn set_first_input(&mut self, id: NodeId) {
        self.first_input = Some(id);
    }

    pub fn get_text_icon_asset_id(&self) -> AssetId {
        *self.text_icon_asset_id_opt.as_ref().unwrap()
    }

    pub fn set_text_icon_asset_id(&mut self, text_icon_asset_id: &AssetId) -> &mut Self {
        self.text_icon_asset_id_opt = Some(text_icon_asset_id.clone());
        self
    }

    pub fn get_text_color(&self) -> Color {
        self.text_color
    }

    pub fn set_text_color(&mut self, text_color: Color) -> &mut Self {
        self.text_color = text_color;
        self
    }

    pub fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_id_map.get(id_str).cloned()
    }
}
