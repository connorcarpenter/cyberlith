use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};
use bevy_log::{info, warn};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use vortex_proto::{FileExtension, resources::FileEntryKey, types::TabId};

use crate::{
    files::{post_process_networked_entities, FileReadOutput, MeshReader, ShapeType, SkelReader},
    resources::{project::Project, ShapeManager, ShapeWaitlist},
};
use crate::files::FileReader;

#[derive(Clone)]
pub struct ContentEntityData {
    shape_type: ShapeType,
}

impl ContentEntityData {
    pub fn new(shape_type: ShapeType) -> Self {
        Self { shape_type }
    }
}

pub struct FileSpace {
    room_key: RoomKey,
    file_entity: Entity,
    content_entities: HashMap<Entity, ContentEntityData>,
    user_count: usize,
}

impl FileSpace {
    pub fn new(
        commands: &mut Commands,
        server: &mut Server,
        shape_waitlist: &mut ShapeWaitlist,
        shape_manager: &mut ShapeManager,
        file_extension: &FileExtension,
        file_room_key: &RoomKey,
        tab_id: TabId,
        bytes: Box<[u8]>,
    ) -> Self {

        let content_entities = Self::load_content_entities(
            commands,
            server,
            shape_waitlist,
            shape_manager,
            file_extension,
            file_room_key,
            tab_id,
            bytes,
        );

        Self {
            room_key,
            file_entity,
            content_entities,
            user_count: 0,
        }
    }

    pub fn room_key(&self) -> RoomKey {
        self.room_key
    }

    pub fn file_entity(&self) -> Entity {
        self.file_entity
    }

    pub fn content_entities(&self) -> &HashMap<Entity, ContentEntityData> {
        &self.content_entities
    }

    fn load_content_entities(
        commands: &mut Commands,
        server: &mut Server,
        shape_waitlist: &mut ShapeWaitlist,
        shape_manager: &mut ShapeManager,
        file_extension: &FileExtension,
        file_room_key: &RoomKey,
        tab_id: TabId,
        bytes: Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {

        // FileReader reads File's contents and spawns all Entities + Components
        let read_output = file_extension.read(commands, server, &bytes);

        let new_entities = match read_output {
            FileReadOutput::Skel(entities) => {
                SkelReader::post_process_entities(shape_waitlist, shape_manager, entities)
            }
            FileReadOutput::Mesh(shape_entities) => {
                MeshReader::post_process_entities(shape_manager, shape_entities)
            }
        };

        post_process_networked_entities(
            commands,
            server,
            file_room_key,
            &new_entities,
            tab_id,
            &file_extension,
        );

        new_entities
    }

    pub(crate) fn user_join(&mut self, server: &mut Server, user_key: &UserKey) {

        self.user_count += 1;

        // put user in new room
        server.room_mut(&self.room_key).add_user(user_key);
    }

    pub fn add_content_entity(&mut self, entity: Entity, shape_type: ShapeType) {
        info!("FileSpace adding content entity: `{:?}`", entity);
        self.content_entities.insert(entity, ContentEntityData::new(shape_type));
    }

    pub fn remove_content_entity(&mut self, entity: &Entity) {
        info!("FileSpace removing content entity: `{:?}`", entity);
        self.content_entities.remove(entity);
    }

    pub fn respawn_content_entities(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        shape_waitlist: &mut ShapeWaitlist,
        shape_manager: &mut ShapeManager,
        file_extension: &FileExtension,
        file_key: &FileEntryKey,
        tab_id: TabId,
        bytes: Box<[u8]>,
    ) {
        if !file_extension.can_io() {
            panic!("can't read file: `{:?}`", file_key.name());
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
                    shape_manager.on_delete_edge(entity);
                }
                ShapeType::Face => {}
            }
        }

        // respawn all entities
        self.content_entities = Self::load_content_entities(
            commands,
            server,
            shape_waitlist,
            shape_manager,
            file_extension,
            &self.room_key,
            tab_id,
            bytes
        );
    }
}

// from usertabstate
// pub fn current_tab_file_entity(&self) -> Option<Entity> {
//     if let Some(tab_id) = self.current_tab {
//         if let Some(state) = self.tabs.get(&tab_id) {
//             Some(state.get_file_entity())
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
//
// pub fn current_tab_file_key(&self) -> Option<Entity> {
//     if let Some(tab_id) = self.current_tab {
//         if let Some(data) = self.tabs.get(&tab_id) {
//             Some(data.get_file_entity())
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
//
// pub(crate) fn current_tab_entities(&self) -> Option<&HashMap<Entity, ContentEntityData>> {
//     if let Some(tab_id) = self.current_tab {
//         self.tab_entities(&tab_id)
//     } else {
//         None
//     }
// }
//
// pub(crate) fn tab_entities(
//     &self,
//     tab_id: &TabId,
// ) -> Option<&HashMap<Entity, ContentEntityData>> {
//     if let Some(state) = self.tabs.get(tab_id) {
//         Some(&state.content_entities)
//     } else {
//         None
//     }
// }
//
// pub(crate) fn current_tab_add_content_entity(
//     &mut self,
//     entity: &Entity,
//     shape_type: ShapeType,
// ) {
//     if let Some(tab_id) = self.current_tab {
//         if let Some(state) = self.tabs.get_mut(&tab_id) {
//             state.add_content_entity(*entity, shape_type);
//         } else {
//             warn!("tab_id {:?} has no state", tab_id);
//         }
//     } else {
//         warn!("no current tab!")
//     }
// }
//
// pub(crate) fn current_tab_remove_content_entity(&mut self, entity: &Entity) {
//     if let Some(tab_id) = self.current_tab {
//         if let Some(state) = self.tabs.get_mut(&tab_id) {
//             state.remove_content_entity(entity);
//         }
//     }
// }
//
//