use std::collections::{HashMap, HashSet};

use bevy_ecs::{entity::Entity, system::Commands};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, RoomKey, Server};
use vortex_proto::{resources::FileEntryKey, types::TabId};

use crate::resources::workspace::Workspace;

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

    pub fn remove_tab_state(&mut self, tab_id: &TabId) -> Option<TabState> {
        if let Some(state) = self.tabs.remove(tab_id) {
            let file_entity = state.file_entity;
            self.file_entity_to_tab_id.remove(&file_entity);
            Some(state)
        } else {
            None
        }
    }

    pub fn insert_tab_state(&mut self, tab_id: TabId, state: TabState) {
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

    pub(crate) fn respawn_tab_content_entities(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        workspace: &Workspace,
        file_entity: &Entity,
        file_entry_key: &FileEntryKey,
    ) {
        if let Some(tab_id) = self.file_entity_to_tab_id.get(file_entity) {
            let Some(tab_state) = self.tabs.get_mut(&tab_id) else {
                panic!("tab_id {:?} has no state", tab_id);
            };
            let tab_is_selected = self.current_tab == Some(*tab_id);
            tab_state.respawn_content_entities(
                commands,
                server,
                workspace,
                file_entry_key,
                tab_is_selected,
            );
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

    pub fn respawn_content_entities(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        workspace: &Workspace,
        file_entry_key: &FileEntryKey,
        tab_is_selected: bool,
    ) {
        if !workspace.working_file_extension(file_entry_key).can_io() {
            panic!("can't read file: `{:?}`", file_entry_key.name());
        }

        // despawn all previous entities
        for entity in self.content_entities.iter() {
            info!("despawning entity: {:?}", entity);
            commands.entity(*entity).take_authority(server).despawn();
        }

        // respawn all entities
        let new_content_entities = workspace.load_content_entities(commands, server, &file_entry_key);

        for entity in new_content_entities.iter() {
            // associate all new Entities with the new Room
            server.room_mut(&self.room_key).add_entity(entity);

            if !tab_is_selected {
                commands
                    .entity(*entity)
                    // call "pause_replication" on all Entities (they will be resumed when tab is selected)
                    .pause_replication(server);
            }
        }

        // update content entities in TabState
        self.content_entities = new_content_entities;
    }
}