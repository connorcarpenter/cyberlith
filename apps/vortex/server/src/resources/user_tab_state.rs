use std::collections::{HashMap, HashSet};

use bevy_ecs::entity::Entity;

use naia_bevy_server::RoomKey;
use vortex_proto::types::TabId;

pub struct UserTabState {
    current_tab: Option<TabId>,
    tabs: HashMap<TabId, TabState>,
    file_entity_to_tab_id: HashMap<Entity, TabId>,
}

impl Default for UserTabState {
    fn default() -> Self {
        Self {
            current_tab: None,
            tabs: HashMap::new(),
            file_entity_to_tab_id: HashMap::new(),
        }
    }
}

impl UserTabState {

    pub fn has_tabs(&self) -> bool {
        !self.tabs.is_empty()
    }

    pub fn state_tabs_remove(&mut self, tab_id: &TabId) -> Option<TabState> {
        if let Some(state) = self.tabs.remove(tab_id) {
            let file_entity = state.file_entity;
            self.file_entity_to_tab_id.remove(&file_entity);
            Some(state)
        } else {
            None
        }
    }

    pub fn state_tabs_insert(&mut self, tab_id: TabId, state: TabState) {
        let file_entity = state.file_entity;
        self.tabs.insert(tab_id, state);
        self.file_entity_to_tab_id.insert(file_entity, tab_id);
    }

    pub fn has_tab_id(&self, tab_id: &TabId) -> bool {
        self.tabs.contains_key(tab_id)
    }

    pub fn set_current_tab(&mut self, tab_id_opt: Option<TabId>) {
        self.current_tab = tab_id_opt;
    }

    pub fn get_current_tab(&self) -> Option<TabId> {
        self.current_tab
    }

    pub fn current_tab_file_entity(&self) -> Option<Entity> {
        if let Some(tab_id) = self.current_tab {
            if let Some(state) = self.tabs.get(&tab_id) {
                Some(state.get_file_entity())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn current_tab_entities(&self) -> Option<&HashSet<Entity>> {
        if let Some(tab_id) = self.current_tab {
            self.tab_entities(&tab_id)
        } else {
            None
        }
    }

    pub(crate) fn tab_entities(&self, tab_id: &TabId) -> Option<&HashSet<Entity>> {
        if let Some(state) = self.tabs.get(tab_id) {
            Some(&state.content_entities)
        } else {
            None
        }
    }

    pub(crate) fn current_tab_add_entity(&mut self, entity: &Entity) {
        if let Some(tab_id) = self.current_tab {
            if let Some(state) = self.tabs.get_mut(&tab_id) {
                state.add_content_entity(*entity);
            }
        }
    }

    pub(crate) fn current_tab_remove_entity(&mut self, entity: &Entity) {
        if let Some(tab_id) = self.current_tab {
            if let Some(state) = self.tabs.get_mut(&tab_id) {
                state.remove_content_entity(entity);
            }
        }
    }
}

pub struct TabState {
    room_key: RoomKey,
    file_entity: Entity,
    content_entities: HashSet<Entity>,
}

impl TabState {
    pub fn new(room_key: RoomKey, file_entity: Entity, content_entities: HashSet<Entity>) -> Self {
        Self {
            room_key,
            file_entity,
            content_entities,
        }
    }

    pub fn add_content_entity(&mut self, entity: Entity) {
        // info!("TabState adding entity: {:?}", entity);
        self.content_entities.insert(entity);
    }

    pub fn remove_content_entity(&mut self, entity: &Entity) {
        // info!("TabState removing entity: {:?}", entity);
        self.content_entities.remove(entity);
    }

    pub fn get_room_key(&self) -> RoomKey {
        self.room_key
    }

    pub fn get_file_entity(&self) -> Entity {
        self.file_entity
    }

    pub fn get_content_entities(&self) -> &HashSet<Entity> {
        &self.content_entities
    }
}