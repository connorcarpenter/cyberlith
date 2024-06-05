use std::{hash::Hash, collections::{HashMap, HashSet}};

use ui_runner::{UiHandle, UiManager, config::{NodeId, UiRuntimeConfig, StyleId}};

pub struct ListUiExt<K: Hash + Eq + Copy + Clone + PartialEq> {
    container_ui: Option<(UiHandle, String)>,
    loaded_items: HashMap<K, HashSet<NodeId>>,
}

impl<K: Hash + Eq + Copy + Clone + PartialEq> ListUiExt<K> {
    pub fn new() -> Self {
        Self {
            container_ui: None,
            loaded_items: HashMap::new(),
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
        Q: 'a + Into<K> + Copy,
        V: 'a,
        C: IntoIterator<Item = (&'a Q, &'a V)>,
        FM: FnMut(&mut ListUiExtItem<K>, K, &'a V, bool),
    > (
        &mut self,
        ui_manager: &mut UiManager,
        collection: C,
        mut item_fn: FM,
    ) {
        if self.container_ui.is_none() {
            return;
        }

        // // remove all node children from list ui
        // {
        //     let (container_ui_handle, container_id_str) = self.container_ui.as_ref().unwrap();
        //     let container_ui_runtime = ui_manager.ui_runtimes.get_mut(container_ui_handle).unwrap();
        //     let container_id = container_ui_runtime.get_node_id_by_id_str(container_id_str).unwrap();
        //     let mut panel_mut = container_ui_runtime.panel_mut(&container_id).unwrap();
        //     panel_mut.remove_all_children();
        // }

        // add new node children to list ui
        {
            let (container_ui_handle, container_ui_str) = self.container_ui.as_ref().unwrap();
            let container_ui_handle = *container_ui_handle;
            let container_ui_runtime = ui_manager.ui_runtimes.get_mut(&container_ui_handle).unwrap();
            let container_id = container_ui_runtime.get_node_id_by_id_str(container_ui_str).unwrap();

            let data_collection_iter = collection.into_iter();
            for (data_key, data_val) in data_collection_iter {
                let data_key = (*data_key).into();

                if self.loaded_items.contains_key(&data_key) {
                    let mut item_mut = ListUiExtItem::new(data_key, self, ui_manager, &container_ui_handle, &container_id);

                    item_fn(&mut item_mut, data_key, data_val, false);
                } else {
                    let mut item_mut = ListUiExtItem::new(data_key, self, ui_manager, &container_ui_handle, &container_id);

                    item_fn(&mut item_mut, data_key, data_val, true);
                }
            }
        }

        // TODO: handle deletion of items

        // queue ui for sync
        ui_manager.queue_recalculate_layout();
        ui_manager.queue_ui_for_sync(self.container_ui.as_ref().map(|(handle, _id_str)| handle).unwrap());
    }
}

pub struct ListUiExtItem<'a, K: Hash + Eq + Copy + Clone + PartialEq> {
    item_key: K,
    list_ui_ext: &'a mut ListUiExt<K>,
    ui_manager: &'a mut UiManager,
    id_str_to_node_map: HashMap<String, NodeId>,
    container_ui_handle: &'a UiHandle,
    container_id: &'a NodeId,
}

impl<'a, K: Hash + Eq + Copy + Clone + PartialEq> ListUiExtItem<'a, K> {
    pub fn new(
        item_key: K,
        list_ui_ext: &'a mut ListUiExt<K>,
        ui_manager: &'a mut UiManager,
        container_ui_handle: &'a UiHandle,
        container_id: &'a NodeId,
    ) -> Self {
        Self {
            item_key,
            list_ui_ext,
            ui_manager,
            id_str_to_node_map: HashMap::new(),
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
        let new_node_id = self.ui_manager.add_copied_node(
            &mut self.id_str_to_node_map,
            self.container_ui_handle,
            self.container_id,
            item_ui_handle,
            &UiRuntimeConfig::ROOT_NODE_ID,
        );

        self.list_ui_ext.loaded_items.entry(self.item_key).or_insert_with(HashSet::new).insert(new_node_id);
    }

    pub fn get_node_id_by_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_map.get(id_str).copied()
    }

    pub fn set_text(&mut self, node_id: &NodeId, text_str: &str) {
        let container_ui_runtime = self.ui_manager.ui_runtimes.get_mut(self.container_ui_handle).unwrap();
        container_ui_runtime.set_text(node_id, text_str);
    }
}