use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};
use bevy_log::{info, warn};

use naia_bevy_server::{CommandsExt, RoomKey, Server};

use vortex_proto::{resources::FileEntryKey, types::TabId};

use crate::{
    files::{post_process_networked_entities, FileReadOutput, MeshReader, ShapeType, SkelReader},
    resources::{ShapeWaitlist, workspace::Workspace, ShapeManager},
};

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

    pub(crate) fn current_tab_entities(&self) -> Option<&HashMap<Entity, ContentEntityData>> {
        if let Some(tab_id) = self.current_tab {
            self.tab_entities(&tab_id)
        } else {
            None
        }
    }

    pub(crate) fn tab_entities(
        &self,
        tab_id: &TabId,
    ) -> Option<&HashMap<Entity, ContentEntityData>> {
        if let Some(state) = self.tabs.get(tab_id) {
            Some(&state.content_entities)
        } else {
            None
        }
    }

    pub(crate) fn current_tab_add_content_entity(&mut self, entity: &Entity, shape_type: ShapeType) {
        if let Some(tab_id) = self.current_tab {
            if let Some(state) = self.tabs.get_mut(&tab_id) {
                state.add_content_entity(*entity, shape_type);
            } else {
                warn!("tab_id {:?} has no state", tab_id);
            }
        } else {
            warn!("no current tab!")
        }
    }

    pub(crate) fn current_tab_remove_content_entity(&mut self, entity: &Entity) {
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
        vertex_waitlist: &mut ShapeWaitlist,
        shape_manager: &mut ShapeManager,
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
                vertex_waitlist,
                shape_manager,
                file_entry_key,
                *tab_id,
                tab_is_selected,
            );
        }
    }
}

#[derive(Clone)]
pub struct ContentEntityData {
    shape_type: ShapeType,
}

impl ContentEntityData {
    pub fn new(shape_type: ShapeType) -> Self {
        Self { shape_type }
    }
}

pub struct TabState {
    room_key: RoomKey,
    file_entity: Entity,
    content_entities: HashMap<Entity, ContentEntityData>,
}

impl TabState {
    pub fn new(
        room_key: RoomKey,
        file_entity: Entity,
        content_entities: HashMap<Entity, ContentEntityData>,
    ) -> Self {
        Self {
            room_key,
            file_entity,
            content_entities,
        }
    }

    pub fn add_content_entity(&mut self, entity: Entity, shape_type: ShapeType) {
        info!("TabState adding content entity: `{:?}`", entity);
        self.content_entities
            .insert(entity, ContentEntityData::new(shape_type));
    }

    pub fn remove_content_entity(&mut self, entity: &Entity) {
        info!("TabState removing content entity: `{:?}`", entity);
        self.content_entities.remove(entity);
    }

    pub fn get_room_key(&self) -> RoomKey {
        self.room_key
    }

    pub fn get_file_entity(&self) -> Entity {
        self.file_entity
    }

    pub fn get_content_entities(&self) -> &HashMap<Entity, ContentEntityData> {
        &self.content_entities
    }

    pub fn respawn_content_entities(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        workspace: &Workspace,
        vertex_waitlist: &mut ShapeWaitlist,
        shape_manager: &mut ShapeManager,
        file_entry_key: &FileEntryKey,
        tab_id: TabId,
        tab_is_selected: bool,
    ) {
        let working_file_extension = workspace.working_file_extension(file_entry_key);
        if !working_file_extension.can_io() {
            panic!("can't read file: `{:?}`", file_entry_key.name());
        }

        // despawn all previous entities
        for (entity, entity_data) in self.content_entities.iter() {
            info!("despawning entity: {:?}", entity);
            commands.entity(*entity).take_authority(server).despawn();

            match entity_data.shape_type {
                ShapeType::Vertex => {
                    shape_manager.on_delete_vertex(commands, server, entity);
                }
                ShapeType::Edge => {
                    //shape_manager.on_delete_edge();
                }
                ShapeType::Face => {}
            }
        }

        // respawn all entities
        let output = workspace.load_content_entities(commands, server, &file_entry_key);

        let new_content_entities = match output {
            FileReadOutput::Skel(entities) => {
                SkelReader::post_process_entities(vertex_waitlist, shape_manager, entities)
            }
            FileReadOutput::Mesh(shape_entities) => {
                MeshReader::post_process_entities(shape_manager, shape_entities)
            }
        };

        post_process_networked_entities(
            commands,
            server,
            &self.room_key,
            &new_content_entities,
            tab_id,
            &working_file_extension,
            !tab_is_selected,
        );

        // update content entities in TabState
        self.content_entities = new_content_entities;
    }
}
