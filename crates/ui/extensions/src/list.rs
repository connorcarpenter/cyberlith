use std::collections::HashMap;

use logging::info;
use ui_runner::{UiHandle, UiManager, UiRuntime, config::{NodeId, StyleId}};

pub struct ListUiExt {
    container_ui: Option<(UiHandle, String)>,
    item_ui: Option<UiHandle>,
    setup_ran: bool,
    stylemap_item_to_list: HashMap<StyleId, StyleId>,
    items_id_str_to_node_id_map: Vec<HashMap<String, NodeId>>,
}

impl ListUiExt {
    pub fn new() -> Self {
        Self {
            container_ui: None,
            item_ui: None,
            setup_ran: false,
            stylemap_item_to_list: HashMap::new(),
            items_id_str_to_node_id_map: Vec::new(),
        }
    }

    pub fn set_container_ui(&mut self, ui_manager: &mut UiManager, ui_handle: &UiHandle, id_str: &str) {
        if self.container_ui.is_some() {
            panic!("container ui already set!");
        }
        self.container_ui = Some((*ui_handle, id_str.to_string()));

        if !self.setup_ran {
            if self.all_uis_loaded() {
                self.setup_list(ui_manager);
            }
        }
    }

    pub fn set_item_ui(&mut self, ui_manager: &mut UiManager,ui_handle: &UiHandle) {
        if self.item_ui.is_some() {
            panic!("item ui already set!");
        }
        self.item_ui = Some(*ui_handle);

        if !self.setup_ran {
            if self.all_uis_loaded() {
                self.setup_list(ui_manager);
            }
        }
    }

    fn all_uis_loaded(&self) -> bool {
        self.container_ui.is_some() && self.item_ui.is_some()
    }

    fn setup_list(&mut self, ui_manager: &mut UiManager) {

        // validate container ui
        let (container_ui_handle, container_id_str) = self.container_ui.as_ref().unwrap();
        if !ui_manager.ui_runtimes.contains_key(container_ui_handle) {
            panic!("container ui not loaded yet!");
        }
        if !ui_manager.ui_has_node_with_id_str(container_ui_handle, container_id_str) {
            panic!("container ui does not have node with id_str: {}", container_id_str);
        }

        // validate item ui
        let item_ui_handle = self.item_ui.as_ref().unwrap();
        if !ui_manager.ui_runtimes.contains_key(item_ui_handle) {
            panic!("item ui not loaded yet!");
        }

        info!("Setting up list ui extension: container_ui={:?}, item_ui={:?}", container_ui_handle, item_ui_handle);

        // make stylemap from item ui to list ui
        let item_ui_runtime = ui_manager.ui_runtimes.get(item_ui_handle).unwrap();
        let item_ui_config = item_ui_runtime.ui_config_ref();

        let mut item_styles = Vec::new();
        for (item_style_id, item_style) in item_ui_config.styles_iter().enumerate() {
            item_styles.push((StyleId::new(item_style_id as u32), *item_style));
        }

        let container_ui_runtime = ui_manager.ui_runtimes.get_mut(container_ui_handle).unwrap();
        for (item_style_id, item_style) in item_styles {

            let list_style_id = container_ui_runtime.add_style(item_style);

            self.stylemap_item_to_list.insert(item_style_id, list_style_id);
        }

        // queue ui layout for recalculation
        ui_manager.queue_recalculate_layout();

        // mark setup as ran
        self.setup_ran = true;
    }

    pub fn sync_with_collection<
        'a,
        K: 'a,
        V: 'a,
        C: IntoIterator<Item = (&'a K, &'a V)>,
        F: FnMut(&mut UiRuntime, &HashMap<String, NodeId>, &'a K, &'a V)
    > (
        &mut self,
        ui_manager: &mut UiManager,
        collection: C,
        mut process_item_fn: F,
    ) {
        if !self.setup_ran {
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

        // // get list of item nodes from item ui
        // // TODO: preload this data
        // let item_root_node = {
        //
        //     let item_ui_runtime = ui_manager.ui_runtimes.get(item_ui_handle).unwrap();
        //     let item_ui_config = item_ui_runtime.ui_config_ref();
        //     item_ui_config.get_node(&UiRuntimeConfig::ROOT_NODE_ID).unwrap()
        //
        //     // for (node_id, node) in item_ui_config.nodes_iter() {
        //     //     let mut new_node = node.clone();
        //     //     if let Some(old_node_style_id) = new_node.style_id() {
        //     //
        //     //         let new_node_style_id = self.stylemap_item_to_list.get(&old_node_style_id).unwrap();
        //     //
        //     //         new_node.clear_style_id();
        //     //         new_node.set_style_id(*new_node_style_id);
        //     //         // info!("Mapped style from item ui to list ui: {:?} -> {:?}", old_node_style_id, new_node_style_id);
        //     //     }
        //     //     item_nodes.push((node_id, new_node));
        //     // }
        // };

        // add new node children to list ui
        {
            let (container_ui_handle, container_ui_str) = self.container_ui.as_ref().unwrap();
            let container_ui_handle = *container_ui_handle;
            let item_ui_handle = *(self.item_ui.as_ref().unwrap());
            let container_ui_runtime = ui_manager.ui_runtimes.get(&container_ui_handle).unwrap();
            let container_id = container_ui_runtime.get_node_id_by_id_str(container_ui_str).unwrap();

            let data_collection_iter = collection.into_iter();
            for (data_key, data_val) in data_collection_iter {
                let mut id_str_map = HashMap::new();

                ui_manager.add_copied_node(&self.stylemap_item_to_list, &mut id_str_map, &container_ui_handle, &container_id, &item_ui_handle, &NodeId::new(0));

                    // info!("Added new node to list ui: {:?}. Total len is: {}", new_node_id, list_ui_config_root_panel.children.len());
                //
                //
                // // update children of panels
                // for (_, new_node_id) in &old_new_id_map {
                //     let Some(panel_mut) = container_ui_runtime_mut.ui_config_mut().panel_mut(new_node_id) else {
                //         continue;
                //     };
                //     // TODO: make this work for button type!
                //     for child_id in panel_mut.children.iter_mut() {
                //         if let Some(new_child_id) = old_new_id_map.get(child_id) {
                //             // info!("Updating child id from {:?} to {:?}", child_id, new_child_id);
                //             *child_id = *new_child_id;
                //         }
                //     }
                // }

                let container_ui_runtime_mut = ui_manager.ui_runtimes.get_mut(&container_ui_handle).unwrap();
                process_item_fn(container_ui_runtime_mut, &id_str_map, data_key, data_val);

                self.items_id_str_to_node_id_map.push(id_str_map);
            }
        }

        // queue ui for sync
        ui_manager.queue_recalculate_layout();
        ui_manager.queue_ui_for_sync(self.container_ui.as_ref().map(|(handle, _id_str)| handle).unwrap());
    }
}