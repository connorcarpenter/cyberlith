use std::collections::HashMap;

use logging::info;
use ui_runner::{UiHandle, UiManager, config::{NodeId, UiRuntimeConfig, StyleId}, UiRuntime};

pub struct ListUiExt {
    container_ui: Option<(UiHandle, String)>,
    copied_styles_old_to_new: HashMap<(UiHandle, StyleId), StyleId>,
    items_id_str_to_node_id_map: Vec<HashMap<String, NodeId>>,
}

impl ListUiExt {
    pub fn new() -> Self {
        Self {
            container_ui: None,
            copied_styles_old_to_new: HashMap::new(),
            items_id_str_to_node_id_map: Vec::new(),
        }
    }

    pub fn set_container_ui(&mut self, ui_manager: &mut UiManager, ui_handle: &UiHandle, id_str: &str) {
        if self.container_ui.is_some() {
            panic!("container ui already set!");
        }
        self.container_ui = Some((*ui_handle, id_str.to_string()));

        // validate container ui
        let (container_ui_handle, container_id_str) = self.container_ui.as_ref().unwrap();
        if !ui_manager.ui_runtimes.contains_key(container_ui_handle) {
            panic!("container ui not loaded yet!");
        }
        if !ui_manager.ui_has_node_with_id_str(container_ui_handle, container_id_str) {
            panic!("container ui does not have node with id_str: {}", container_id_str);
        }

        // queue ui layout for recalculation
        ui_manager.queue_recalculate_layout();
    }

    pub fn sync_with_collection<
        'a,
        K: 'a,
        V: 'a,
        C: IntoIterator<Item = (&'a K, &'a V)>,
        F: FnMut(&mut ListUiExtItem, &'a K, &'a V)
    > (
        &mut self,
        ui_manager: &mut UiManager,
        collection: C,
        mut process_item_fn: F,
    ) {
        if self.container_ui.is_none() {
            return;
        }

        // remove node id map
        self.items_id_str_to_node_id_map = Vec::new();

        // remove all node children from list ui
        {
            let (container_ui_handle, container_id_str) = self.container_ui.as_ref().unwrap();
            let container_ui_runtime = ui_manager.ui_runtimes.get_mut(container_ui_handle).unwrap();
            let container_id = container_ui_runtime.get_node_id_by_id_str(container_id_str).unwrap();
            let mut panel_mut = container_ui_runtime.panel_mut(&container_id).unwrap();
            panel_mut.remove_all_children();
        }

        // add new node children to list ui
        {
            let (container_ui_handle, container_ui_str) = self.container_ui.as_ref().unwrap();
            let container_ui_handle = *container_ui_handle;
            let container_ui_runtime = ui_manager.ui_runtimes.get(&container_ui_handle).unwrap();
            let container_id = container_ui_runtime.get_node_id_by_id_str(container_ui_str).unwrap();

            let data_collection_iter = collection.into_iter();
            for (data_key, data_val) in data_collection_iter {
                let mut id_str_map = HashMap::new();

                let mut item_mut = ListUiExtItem::new(ui_manager, self, &mut id_str_map, &container_ui_handle, &container_id);

                process_item_fn(&mut item_mut, data_key, data_val);

                self.items_id_str_to_node_id_map.push(id_str_map);
            }
        }

        // queue ui for sync
        ui_manager.queue_recalculate_layout();
        ui_manager.queue_ui_for_sync(self.container_ui.as_ref().map(|(handle, _id_str)| handle).unwrap());
    }
}

pub struct ListUiExtItem<'a> {
    ui_manager: &'a mut UiManager,
    list_ext: &'a mut ListUiExt,
    id_str_to_node_map: &'a mut HashMap<String, NodeId>,
    container_ui_handle: &'a UiHandle,
    container_id: &'a NodeId,
}

impl<'a> ListUiExtItem<'a> {
    pub fn new(
        ui_manager: &'a mut UiManager,
        list_ext: &'a mut ListUiExt,
        id_str_to_node_map: &'a mut HashMap<String, NodeId>,
        container_ui_handle: &'a UiHandle,
        container_id: &'a NodeId,
    ) -> Self {
        Self {
            ui_manager,
            list_ext,
            id_str_to_node_map,
            container_ui_handle,
            container_id,
        }
    }

    pub fn add_copied_node(&mut self, item_ui_handle: &UiHandle) {

        // add styles if needed
        {
            let container_ui_runtime = self.ui_manager.ui_runtimes.get(self.container_ui_handle).unwrap();
            if !container_ui_runtime.has_copied_style(item_ui_handle) {
                // make stylemap from item ui to list ui
                let item_ui_config = self.ui_manager.ui_runtimes.get(item_ui_handle).unwrap().ui_config_ref();

                let mut item_styles = Vec::new();
                for (item_style_id, item_style) in item_ui_config.styles_iter().enumerate() {
                    item_styles.push((StyleId::new(item_style_id as u32), *item_style));
                }

                let container_ui_runtime = self.ui_manager.ui_runtimes.get_mut(self.container_ui_handle).unwrap();

                for (old_style_id, item_style) in item_styles {
                    container_ui_runtime.add_copied_style(item_ui_handle, old_style_id, item_style);
                }
            }
        }

        // add node
        self.ui_manager.add_copied_node(
            self.id_str_to_node_map,
            self.container_ui_handle,
            self.container_id,
            item_ui_handle,
            &UiRuntimeConfig::ROOT_NODE_ID,
        );
    }

    pub fn get_node_id_by_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_map.get(id_str).copied()
    }

    pub fn set_text(&mut self, node_id: &NodeId, text_str: &str) {
        let container_ui_runtime = self.ui_manager.ui_runtimes.get_mut(self.container_ui_handle).unwrap();
        container_ui_runtime.set_text(node_id, text_str);
    }
}