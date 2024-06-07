use std::{hash::Hash, collections::{HashMap, HashSet}};

use logging::info;

use ui_runner::{UiHandle, UiManager, config::{NodeId, UiRuntimeConfig, StyleId}};

pub struct LoadedItem {
    node_ids: HashSet<NodeId>,
    id_str_to_node_map: HashMap<String, NodeId>,
    old_actions: Vec<ListItemAction>,
}

impl LoadedItem {
    pub fn new(old_actions: Vec<ListItemAction>) -> Self {
        Self {
            node_ids: HashSet::new(),
            id_str_to_node_map: HashMap::new(),
            old_actions,
        }
    }

    pub fn add_node(&mut self, node_id: NodeId) {
        self.node_ids.insert(node_id);
    }

    pub fn nodes_len(&self) -> usize {
        self.node_ids.len()
    }

    pub(crate) fn actions_are_equal(&self, new_actions: &Vec<ListItemAction>) -> bool {
        if self.old_actions.len() != new_actions.len() {
            return false;
        }
        for i in 0..self.old_actions.len() {
            if self.old_actions[i] != new_actions[i] {
                return false;
            }
        }
        return true;
    }

    pub fn deconstruct(self) -> (HashSet<NodeId>, HashMap<String, NodeId>) {
        (self.node_ids, self.id_str_to_node_map)
    }
}

pub struct ListUiExt<K: Hash + Eq + Copy + Clone + PartialEq> {
    container_ui: Option<(UiHandle, String)>,
    loaded_items: HashMap<K, LoadedItem>,
    item_count: usize,
    visible_item_min_index: usize,
    visible_item_max_index: usize,
    visible_item_range: usize,
}

impl<K: Hash + Eq + Copy + Clone + PartialEq> ListUiExt<K> {
    pub fn new(_top_align: bool) -> Self {
        Self {
            container_ui: None,
            loaded_items: HashMap::new(),
            item_count: 0,
            visible_item_min_index: 0,
            visible_item_max_index: 19,
            visible_item_range: 20,
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

    pub fn scroll_up(&mut self) {
        if self.visible_item_min_index > 0 {
            info!("scroll_up");
            self.visible_item_min_index -= 1;
            self.visible_item_max_index -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.visible_item_max_index < self.item_count - 1 {
            info!("scroll_down");
            self.visible_item_min_index += 1;
            self.visible_item_max_index += 1;
        }
    }

    pub fn sync_with_collection<
        'a,
        Q: 'a + Into<K> + Copy,
        V: 'a,
        C: Iterator<Item = (&'a Q, &'a V)>,
        FM: FnMut(&mut ListUiExtItem<K>, K, &'a V),
    > (
        &mut self,
        ui_manager: &mut UiManager,
        collection: C,
        item_count: usize,
        mut item_fn: FM,
    ) {
        if self.container_ui.is_none() {
            return;
        }
        self.item_count = item_count;

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

            let mut item_index = 0;
            let mut item_visible_index = 0;

            for (data_key, data_val) in collection {

                let data_key = (*data_key).into();

                if self.index_is_in_range(item_index) {
                    let mut item_mut = ListUiExtItem::new(item_visible_index, data_key, self, ui_manager, &container_ui_handle, &container_id);

                    item_fn(&mut item_mut, data_key, data_val);

                    item_mut.finished();

                    let loaded_nodes = self.loaded_items.get(&data_key).unwrap().nodes_len();
                    item_visible_index += loaded_nodes;
                } else {
                    if self.loaded_items.contains_key(&data_key) {
                        let container_ui_runtime = ui_manager.ui_runtimes.get_mut(&container_ui_handle).unwrap();
                        let (item_nodes, _) = self.loaded_items.remove(&data_key).unwrap().deconstruct();
                        for item_node in item_nodes {
                            // remove from main panel
                            container_ui_runtime.panel_mut(&container_id).unwrap().remove_node(&item_node);

                            // delete
                            container_ui_runtime.delete_node_recurse(&item_node);
                        }
                    }
                }

                item_index += 1;
            }
        }

        // TODO: handle deletion of items

        // queue ui for sync
        ui_manager.queue_recalculate_layout();
        ui_manager.queue_ui_for_sync(self.container_ui.as_ref().map(|(handle, _id_str)| handle).unwrap());
    }

    pub(crate) fn get_id_str_to_node_map_mut(&mut self, item_key: &K) -> &mut HashMap<String, NodeId> {
        let loaded_item = self.loaded_items.get_mut(item_key).unwrap();
        &mut loaded_item.id_str_to_node_map
    }

    pub(crate) fn get_node_id_by_str(&self, item_key: K, node_str: &str) -> Option<&NodeId> {
        self.loaded_items.get(&item_key).and_then(|loaded_item| loaded_item.id_str_to_node_map.get(node_str))
    }

    pub(crate) fn actions_are_equal(&self, item_key: K, new_actions: &Vec<ListItemAction>) -> bool {
        let Some(loaded_item) = self.loaded_items.get(&item_key) else {
            return false;
        };
        return loaded_item.actions_are_equal(new_actions);
    }

    fn index_is_in_range(&self, item_index: usize) -> bool {
        item_index >= self.visible_item_min_index && item_index <= self.visible_item_max_index
    }
}

#[derive(Eq, PartialEq, Clone)]
enum ListItemAction {
    AddCopiedNode(UiHandle),
    SetTextByStr(String, String),
}

pub struct ListUiExtItem<'a, K: Hash + Eq + Copy + Clone + PartialEq> {
    item_visible_index: usize,
    item_key: K,
    list_ui_ext: &'a mut ListUiExt<K>,
    ui_manager: &'a mut UiManager,
    container_ui_handle: &'a UiHandle,
    container_id: &'a NodeId,
    actions: Vec<ListItemAction>,
}

impl<'a, K: Hash + Eq + Copy + Clone + PartialEq> ListUiExtItem<'a, K> {
    pub fn new(
        item_visible_index: usize,
        item_key: K,
        list_ui_ext: &'a mut ListUiExt<K>,
        ui_manager: &'a mut UiManager,
        container_ui_handle: &'a UiHandle,
        container_id: &'a NodeId,
    ) -> Self {
        Self {
            item_visible_index,
            item_key,
            list_ui_ext,
            ui_manager,
            container_ui_handle,
            container_id,
            actions: Vec::new(),
        }
    }

    pub fn add_copied_node(&mut self, item_ui_handle: &UiHandle) {
        self.actions.push(ListItemAction::AddCopiedNode(*item_ui_handle));
    }

    pub fn set_text_by_str(&mut self, id_str: &str, text_str: &str) {
        self.actions.push(ListItemAction::SetTextByStr(id_str.to_string(), text_str.to_string()));
    }

    fn add_copied_node_impl(&mut self, item_ui_handle: &UiHandle) {

        info!("add_copied_node: {:?}", item_ui_handle);

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
        let mut id_str_to_node_map = self.list_ui_ext.get_id_str_to_node_map_mut(&self.item_key);
        let new_node_id = self.ui_manager.insert_copied_node(
            self.item_visible_index,
            &mut id_str_to_node_map,
            self.container_ui_handle,
            self.container_id,
            item_ui_handle,
            &UiRuntimeConfig::ROOT_NODE_ID,
        );
        self.item_visible_index += 1;

        let loaded_item = self.list_ui_ext.loaded_items.get_mut(&self.item_key).unwrap();
        loaded_item.add_node(new_node_id);
    }

    fn set_text_by_str_impl(&mut self, id_str: &str, text_str: &str) {

        let node_id = self.list_ui_ext.get_node_id_by_str(self.item_key, id_str).unwrap();

        let container_ui_runtime = self.ui_manager.ui_runtimes.get_mut(self.container_ui_handle).unwrap();
        container_ui_runtime.set_text(node_id, text_str);
    }

    pub fn finished(mut self) {

        if self.list_ui_ext.actions_are_equal(self.item_key, &self.actions) {
            return;
        }

        if let Some(loaded_item) = self.list_ui_ext.loaded_items.remove(&self.item_key) {
            let container_ui_runtime = self.ui_manager.ui_runtimes.get_mut(self.container_ui_handle).unwrap();
            let (item_nodes, _) = loaded_item.deconstruct();
            for item_node in item_nodes {
                // remove from main panel
                container_ui_runtime.panel_mut(self.container_id).unwrap().remove_node(&item_node);

                // delete
                container_ui_runtime.delete_node_recurse(&item_node);
            }
        }

        let new_actions = std::mem::take(&mut self.actions);

        self.list_ui_ext.loaded_items.insert(self.item_key, LoadedItem::new(new_actions.clone()));

        // execute actions
        for action in new_actions {
            match action {
                ListItemAction::AddCopiedNode(ui_handle) => {
                    self.add_copied_node_impl(&ui_handle);
                }
                ListItemAction::SetTextByStr(id_str, text) => {
                    self.set_text_by_str_impl(&id_str, &text);
                }
            }
        }
    }
}