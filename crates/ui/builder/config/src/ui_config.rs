use std::collections::{btree_map::Iter, BTreeMap, HashMap};

use logging::info;
use ui_layout::NodeId;

use crate::{
    node::UiNode,
    panel::Panel,
    style::{NodeStyle, StyleId},
    widget::Widget,
};

pub struct UiConfig {
    styles: Vec<NodeStyle>,
    nodes: BTreeMap<NodeId, UiNode>,

    next_node_id: NodeId,
    first_input: Option<NodeId>,
    id_str_to_node_id_map: HashMap<String, NodeId>,
}

impl UiConfig {
    pub const ROOT_NODE_ID: NodeId = NodeId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            styles: Vec::new(),
            nodes: BTreeMap::new(),

            next_node_id: NodeId::new(0),
            first_input: None,
            id_str_to_node_id_map: HashMap::new(),
        };

        // Root Node
        let root_panel_id = me.create_node(None, Widget::Panel(Panel::new()));
        if root_panel_id != Self::ROOT_NODE_ID {
            panic!("root panel id is not 0");
        }

        me
    }

    pub fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        let id_u32 = id.as_usize() as u32;
        self.next_node_id = NodeId::new(id_u32 + 1);
        id
    }

    pub fn decompose(
        self,
    ) -> (
        Vec<NodeStyle>,
        BTreeMap<NodeId, UiNode>,
        Option<NodeId>,
        HashMap<String, NodeId>,
    ) {
        (
            self.styles,
            self.nodes,
            self.first_input,
            self.id_str_to_node_id_map,
        )
    }

    // nodes

    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(id)
    }

    pub fn nodes_iter(&self) -> Iter<'_, NodeId, UiNode> {
        self.nodes.iter()
    }

    pub fn create_node(&mut self, id_str_opt: Option<&str>, widget: Widget) -> NodeId {
        let node_id = self.next_node_id();
        let ui_node = UiNode::new(id_str_opt, widget);
        self.insert_node(&node_id, ui_node);

        if let Some(id_str) = id_str_opt {
            // info!("inserting id_str: {} for node_id: {:?}", id_str, node_id);
            self.id_str_to_node_id_map
                .insert(id_str.to_string(), node_id);
        }

        node_id
    }

    fn insert_node(&mut self, id: &NodeId, node: UiNode) {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        self.nodes.insert(*id, node);
    }

    // styles

    pub fn style_mut(&mut self, id: &StyleId) -> Option<&mut NodeStyle> {
        self.styles.get_mut(id.as_usize())
    }

    pub fn styles_iter(&self) -> std::slice::Iter<'_, NodeStyle> {
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

    pub fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_id_map.get(id_str).cloned()
    }
}
