use std::collections::HashMap;
use logging::info;
use ui_runner::{UiHandle, UiManager, UiRuntime};
use ui_runner::config::{NodeId, StyleId, UiRuntimeConfig};

pub struct ListUiExt {
    container_ui: Option<(UiHandle, String)>,
    list_ui: Option<UiHandle>,
    item_ui: Option<UiHandle>,
    setup_ran: bool,
    stylemap_item_to_list: HashMap<StyleId, StyleId>,
    items_id_str_to_node_id_map: Vec<HashMap<String, NodeId>>,
}

impl ListUiExt {
    pub fn new() -> Self {
        Self {
            container_ui: None,
            list_ui: None,
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

    pub fn set_list_ui(&mut self, ui_manager: &mut UiManager,ui_handle: &UiHandle) {
        if self.list_ui.is_some() {
            panic!("list ui already set!");
        }
        self.list_ui = Some(*ui_handle);

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
        self.container_ui.is_some() && self.list_ui.is_some() && self.item_ui.is_some()
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

        // validate list ui
        let list_ui_handle = self.list_ui.as_ref().unwrap();
        if !ui_manager.ui_runtimes.contains_key(list_ui_handle) {
            panic!("list ui not loaded yet!");
        }
        let runtime = ui_manager.ui_runtimes.get(list_ui_handle).unwrap();
        if runtime.ui_config_ref().nodes_len() != 1 {
            panic!("list ui does not have exactly one node!");
        }

        // validate item ui
        let item_ui_handle = self.item_ui.as_ref().unwrap();
        if !ui_manager.ui_runtimes.contains_key(item_ui_handle) {
            panic!("item ui not loaded yet!");
        }

        info!("Setting up list ui extension: container_ui={:?}, list_ui={:?}, item_ui={:?}", container_ui_handle, list_ui_handle, item_ui_handle);

        // add list ui to container's ui_container widget
        ui_manager.set_ui_container_contents(container_ui_handle, container_id_str, list_ui_handle);

        // make stylemap from item ui to list ui
        let item_ui_config = ui_manager.ui_runtimes.get(item_ui_handle).unwrap().ui_config_ref();

        let mut item_styles = Vec::new();
        for (item_style_id, item_style) in item_ui_config.styles_iter().enumerate() {
            item_styles.push((StyleId::new(item_style_id as u32), *item_style));
        }

        let list_ui_runtime = ui_manager.ui_runtimes.get_mut(list_ui_handle).unwrap();
        for (item_style_id, item_style) in item_styles {

            let list_style_id = list_ui_runtime.add_style(item_style);

            self.stylemap_item_to_list.insert(item_style_id, list_style_id);
        }


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
            let list_ui_handle = self.list_ui.as_ref().unwrap();
            let list_ui_runtime = ui_manager.ui_runtimes.get_mut(list_ui_handle).unwrap();
            list_ui_runtime.remove_nodes_after(&UiRuntimeConfig::ROOT_NODE_ID);

            let list_ui_config_root_panel = list_ui_runtime.ui_config_mut().panel_mut(&UiRuntimeConfig::ROOT_NODE_ID).unwrap();
            list_ui_config_root_panel.remove_all_children();
        }

        // get list of item nodes from item ui
        let mut item_nodes = Vec::new();
        {
            let item_ui_handle = self.item_ui.as_ref().unwrap();
            let item_ui_runtime = ui_manager.ui_runtimes.get(item_ui_handle).unwrap();
            let item_ui_config = item_ui_runtime.ui_config_ref();

            for (node_id, node) in item_ui_config.nodes_iter().enumerate().map(|(i, n)| (NodeId::new(i as u32), n)) {
                let mut new_node = node.clone();
                if let Some(new_node_style_id) = new_node.style_id() {

                    let new_node_style_id = self.stylemap_item_to_list.get(&new_node_style_id).unwrap();

                    new_node.clear_style_id();
                    new_node.set_style_id(*new_node_style_id);
                }
                item_nodes.push((node_id, new_node));
            }
        }

        // add new node children to list ui
        {
            let list_ui_handle = self.list_ui.as_ref().unwrap();
            let list_ui_runtime_mut = ui_manager.ui_runtimes.get_mut(list_ui_handle).unwrap();

            let mut data_collection_iter = collection.into_iter();
            for (data_key, data_val) in data_collection_iter {
                let mut id_str_map = HashMap::new();
                let mut old_new_id_map = HashMap::new();

                for (old_node_id, item_node) in &item_nodes {
                    let new_node_id = list_ui_runtime_mut.add_node(item_node.clone());

                    old_new_id_map.insert(*old_node_id, new_node_id);

                    if let Some(id_str) = item_node.widget.id_str_opt() {
                        id_str_map.insert(id_str.to_string(), new_node_id);
                    }

                    let list_ui_config_root_panel = list_ui_runtime_mut.ui_config_mut().panel_mut(&UiRuntimeConfig::ROOT_NODE_ID).unwrap();
                    list_ui_config_root_panel.add_child(new_node_id);
                    // info!("Added new node to list ui: {:?}. Total len is: {}", new_node_id, list_ui_config_root_panel.children.len());
                }

                // update children of panels
                for (_, new_node_id) in &old_new_id_map {
                    let Some(panel_mut) = list_ui_runtime_mut.ui_config_mut().panel_mut(new_node_id) else {
                        continue;
                    };
                    for child_id in panel_mut.children.iter_mut() {
                        if let Some(new_child_id) = old_new_id_map.get(child_id) {
                            *child_id = *new_child_id;
                        }
                    }
                }

                process_item_fn(list_ui_runtime_mut, &id_str_map, data_key, data_val);

                self.items_id_str_to_node_id_map.push(id_str_map);
            }
        }
    }
}