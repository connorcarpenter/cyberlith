use std::collections::HashMap;

use bevy_ecs::entity::Entity;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use bevy_log::info;

use naia_bevy_server::{RoomKey, Server, UserKey};

use vortex_proto::resources::FileKey;

use crate::files::ShapeType;

#[derive(Clone, Debug)]
pub enum ContentEntityData {
    Shape(ShapeType),
    Dependency(FileKey),
    Frame,
    Rotation,
    Color,
}

impl ContentEntityData {
    pub fn new_shape(shape_type: ShapeType) -> Self {
        Self::Shape(shape_type)
    }

    pub fn new_dependency(dependency_key: FileKey) -> Self {
        Self::Dependency(dependency_key)
    }

    pub fn new_frame() -> Self {
        Self::Frame
    }

    pub fn new_rotation() -> Self {
        Self::Rotation
    }

    pub fn new_color() -> Self {
        Self::Color
    }
}

pub struct FileSpace {
    room_key: RoomKey,
    content_entities: HashMap<Entity, ContentEntityData>,
    user_count: usize,
}

impl FileSpace {
    pub fn new(
        file_room_key: &RoomKey,
        content_entities: HashMap<Entity, ContentEntityData>,
    ) -> Self {
        Self {
            room_key: *file_room_key,
            content_entities,
            user_count: 0,
        }
    }

    pub fn room_key(&self) -> RoomKey {
        self.room_key
    }

    pub fn content_entities(&self) -> &HashMap<Entity, ContentEntityData> {
        &self.content_entities
    }

    pub(crate) fn user_join(&mut self, world: &mut World, user_key: &UserKey) {
        self.user_count += 1;

        // put user in room
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let mut server = system_state.get_mut(world);

        if !server.room(&self.room_key).has_user(user_key) {
            server.room_mut(&self.room_key).add_user(user_key);
        }
    }

    pub(crate) fn user_leave(&mut self, server: &mut Server, user_key: &UserKey) {
        self.user_count -= 1;

        // user leave room
        if server.room(&self.room_key).has_user(user_key) {
            server.room_mut(&self.room_key).remove_user(user_key);
        }
    }

    pub(crate) fn has_no_users(&self) -> bool {
        self.user_count == 0
    }

    pub fn add_content_entity(&mut self, entity: Entity, content_data: ContentEntityData) {
        info!(
            "FileSpace adding content entity: `{:?}`, `{:?}`",
            entity, content_data
        );
        self.content_entities.insert(entity, content_data);
    }

    pub fn remove_content_entity(&mut self, entity: &Entity) {
        info!("FileSpace removing content entity: `{:?}`", entity);
        self.content_entities.remove(entity);
    }

    pub fn set_content_entities(&mut self, content_entities: HashMap<Entity, ContentEntityData>) {
        self.content_entities = content_entities;
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
