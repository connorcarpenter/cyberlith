use std::{iter::{Peekable, Rev}, hash::Hash, collections::{HashMap, HashSet}};

use asset_loader::AssetManager;

use ui_runner::{UiHandle, UiManager, config::{NodeId, NodeStore, UiRuntimeConfig, Alignment, StyleId}};

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

trait PeekableIterator {
    type Item;
    fn peek(&mut self) -> Option<&Self::Item>;
    fn next(&mut self) -> Option<Self::Item>;
}

struct PeekableIteratorImpl<I: Iterator>(Peekable<I>);

impl<I: Iterator> PeekableIterator for PeekableIteratorImpl<I> {
    type Item = I::Item;

    fn peek(&mut self) -> Option<&Self::Item> {
        self.0.peek()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

struct RevPeekableIteratorImpl<I: DoubleEndedIterator>(Peekable<Rev<I>>);

impl<I: DoubleEndedIterator> PeekableIterator for RevPeekableIteratorImpl<I> {
    type Item = I::Item;

    fn peek(&mut self) -> Option<&Self::Item> {
        self.0.peek()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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
            self.visible_item_min_index -= 1;
            self.visible_item_max_index -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.visible_item_max_index < self.item_count - 1 {
            self.visible_item_min_index += 1;
            self.visible_item_max_index += 1;
        }
    }

    pub fn sync_with_collection<
        'a,
        Q: 'a + Into<K> + Copy,
        V: 'a,
        C: DoubleEndedIterator<Item = (&'a Q, &'a V)>,
        FM: FnMut(&mut ListUiExtItem<K>, K, Option<K>),
    > (
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        collection: C,
        item_count: usize,
        mut item_fn: FM,
    ) {
        if self.container_ui.is_none() {
            return;
        }

        // recalculate all nodes so that we can get the correct parent container dimensions
        ui_manager.queue_recalculate_layout();
        ui_manager.recalculate_ui_layout_if_needed(asset_manager);

        let (container_ui_handle, container_ui_str) = self.container_ui.as_ref().unwrap();
        let container_ui_handle = *container_ui_handle;
        let container_ui_runtime = ui_manager.ui_runtimes.get_mut(&container_ui_handle).unwrap();
        let container_id = container_ui_runtime.get_node_id_by_id_str(container_ui_str).unwrap();
        let Some((_parent_width, parent_height)) = container_ui_runtime.get_node_dimensions(&container_id) else {
            return;
        };
        let parent_children_valign = container_ui_runtime.ui_config_ref().node_children_valign(&container_id);
        let parent_children_node_count = container_ui_runtime.ui_config_ref().panel_ref(&container_id).unwrap().children.len();

        if self.item_count == 0 {
            // first time sync
            self.visible_item_max_index = item_count - 1;
        }
        self.item_count = item_count;

        // add new node children to list ui
        {
            // what to do here to get the correct item index range??

            let mut item_index;
            let mut current_child_index;
            let mut boxed_iterator: Box<dyn PeekableIterator<Item=(&'a Q, &'a V)>>;
            let mut used_space = 0.0;
            let iterator_incrementing: bool;

            if parent_children_valign == Alignment::End {
                item_index = item_count - 1;
                current_child_index = parent_children_node_count;
                boxed_iterator = Box::new(RevPeekableIteratorImpl(collection.rev().peekable()));
                iterator_incrementing = false;
            } else {
                item_index = 0;
                current_child_index = 0;
                boxed_iterator = Box::new(PeekableIteratorImpl(collection.peekable()));
                iterator_incrementing = true;
            }

            // info!("item_index: {:?}", item_index);

            loop {

                let Some((data_key, _)) = boxed_iterator.next() else {
                    break;
                };
                let next_data_key_opt = boxed_iterator.peek().map(|(data_key, _)| (**data_key).into());

                // info!("item_index: {:?}", item_index);

                let data_key = (*data_key).into();

                if !iterator_incrementing {
                    let loaded_nodes = if let Some(item) = self.loaded_items.get(&data_key) { item.nodes_len() } else { 0 };
                    current_child_index -= loaded_nodes;
                }

                if used_space < parent_height && self.try_to_add_item(item_index, iterator_incrementing) {
                    let mut item_mut = ListUiExtItem::new(
                        current_child_index,
                        &mut used_space,
                        data_key,
                        self,
                        ui_manager,
                        &container_ui_handle,
                        &container_id
                    );

                    item_fn(&mut item_mut, data_key, next_data_key_opt);

                    item_mut.finished(parent_height);

                    // info!("used_space: {:?} / parent_space: {:?}", used_space, parent_height);

                    if used_space > parent_height {
                        // info!("done iterating visible items");
                        if iterator_incrementing {
                            // set visible item max index to current item index
                            self.visible_item_max_index = item_index - 1;
                        } else {
                            // set visible item min index to current item index
                            self.visible_item_min_index = item_index + 1;
                        }
                        self.visible_item_range = self.visible_item_max_index - self.visible_item_min_index + 1;
                        // info!("visible_item_min_index: {:?}, visible_item_max_index: {:?}, visible_item_range: {:?}", self.visible_item_min_index, self.visible_item_max_index, self.visible_item_range);
                    } else {
                        if iterator_incrementing {
                            let loaded_nodes = self.loaded_items.get(&data_key).unwrap().nodes_len();
                            current_child_index += loaded_nodes;
                        }
                    }
                } else {
                    // remove any nodes not visible
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

                if iterator_incrementing {
                    item_index += 1;
                } else {
                    if item_index > 0 {
                        item_index -= 1;
                    } else {
                        break;
                    }
                }
            }
        }

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

    fn try_to_add_item(&self, item_index: usize, iterator_incrementing: bool) -> bool {
        if iterator_incrementing {
            return item_index >= self.visible_item_min_index;
        } else {
            return item_index <= self.visible_item_max_index;
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
enum ListItemAction {
    AddCopiedNode(UiHandle),
    SetTextByStr(String, String),
}

pub struct ListUiExtItem<'a, K: Hash + Eq + Copy + Clone + PartialEq> {
    item_visible_index: usize,
    used_space: &'a mut f32,
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
        used_space: &'a mut f32,
        item_key: K,
        list_ui_ext: &'a mut ListUiExt<K>,
        ui_manager: &'a mut UiManager,
        container_ui_handle: &'a UiHandle,
        container_id: &'a NodeId,
    ) -> Self {
        Self {
            item_visible_index,
            used_space,
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

        // info!("add_copied_node: {:?}", item_ui_handle);

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

    pub fn finished(mut self, parent_height: f32) {

        let should_add: bool;
        let actions_are_equal: bool;

        // should we add a new item? check against used space and parent height
        let item_height = self.get_item_height(parent_height);
        *self.used_space += item_height;
        if *self.used_space > parent_height {
            should_add = false;
        } else {
            should_add = true;
        }

        // check if actions are equal
        actions_are_equal = self.list_ui_ext.actions_are_equal(self.item_key, &self.actions);

        if !should_add || !actions_are_equal {
            // remove old nodes
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
        }

        if should_add && !actions_are_equal {
            // add new nodes
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

    fn get_item_height(&self, parent_height: f32) -> f32 {
        let mut item_height = 0.0;
        for action in &self.actions {
            match action {
                ListItemAction::AddCopiedNode(ui_handle) => {
                    let item_ui_runtime = self.ui_manager.ui_runtimes.get(ui_handle).unwrap();
                    let item_ui_config = item_ui_runtime.ui_config_ref();
                    let item_node_height_su = item_ui_config.node_height(&UiRuntimeConfig::ROOT_NODE_ID);
                    if item_node_height_su.is_auto() {
                        panic!("item node height cannot be auto");
                    }
                    let item_node_height = item_node_height_su.to_px(parent_height, parent_height, 0.0, 0.0);
                    item_height += item_node_height;
                }
                _ => {}
            }
        }
        item_height
    }
}