use ui_runner_config::{NodeId, UiNode};

use crate::UiRuntime;

pub struct PanelMut<'a> {
    runtime: &'a mut UiRuntime,
    node_id: NodeId,
}

impl<'a> PanelMut<'a> {
    pub fn new(runtime: &'a mut UiRuntime, node_id: NodeId) -> Self {
        Self { runtime, node_id }
    }

    pub fn insert_node(&mut self, index: usize, node: &UiNode) -> NodeId {
        let new_node_id = self.runtime.ui_config_mut().add_node(node.clone());
        self.runtime.ui_state_mut().add_node(&new_node_id, &node);
        let parent_panel_mut = self
            .runtime
            .ui_config_mut()
            .panel_mut(&self.node_id)
            .unwrap();
        parent_panel_mut.children.insert(index, new_node_id);
        new_node_id
    }

    pub fn remove_node(&mut self, node_id: &NodeId) {
        let parent_panel_mut = self
            .runtime
            .ui_config_mut()
            .panel_mut(&self.node_id)
            .unwrap();
        parent_panel_mut.children.retain(|id| id != node_id);
    }

    pub fn remove_all_children(&mut self) {
        let panel_mut = self
            .runtime
            .ui_config_mut()
            .panel_mut(&self.node_id)
            .unwrap();
        let child_ids = std::mem::take(&mut panel_mut.children);
        for child_id in child_ids {
            self.runtime.delete_node_recurse(&child_id);
        }
    }
}

//pub fn add_node(&mut self, node: UiNode) -> NodeId {
//         self.state.node_state_init(&node);
//         self.config.add_node(node)
//     }
//
//     pub fn remove_nodes_after(&mut self, node_id: &NodeId) {
//         self.state.remove_nodes_after(node_id);
//         self.config.remove_nodes_after(node_id);
//     }
//
//     pub fn remove_child_nodes_by_id_str(&mut self, id_str: &str) {
//
//     }
//pub fn add_node(&mut self, node: UiNode) -> NodeId {
//         let id = NodeId::from_usize(self.nodes.len());
//         self.nodes.push(node);
//         id
//     }
//
//     pub fn remove_node(&mut self, id: &NodeId) {
//         self.nodes.remove(id);
//     }
