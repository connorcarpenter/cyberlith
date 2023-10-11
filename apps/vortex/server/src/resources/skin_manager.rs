use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct SkinManager {
    // face_3d_entity -> face color entity
    face_to_color_map: HashMap<Entity, Entity>,
    // face color_entity -> face_3d_entity
    color_to_face_map: HashMap<Entity, Entity>,
}

impl Default for SkinManager {
    fn default() -> Self {
        Self {
            face_to_color_map: HashMap::new(),
            color_to_face_map: HashMap::new(),
        }
    }
}

impl SkinManager {
    pub fn has_face_color(&self, face_color_entity: &Entity) -> bool {
        self.color_to_face_map.contains_key(face_color_entity)
    }

    pub fn on_create_face_color(
        &mut self,
        face_3d_entity: &Entity,
        face_color_entity: &Entity,
    ) {
        self.color_to_face_map.insert(*face_color_entity, *face_3d_entity);
        self.face_to_color_map.insert(*face_3d_entity, *face_color_entity);
    }

    pub fn on_despawn_face_color(
        &mut self,
        face_color_entity: &Entity,
    ) {
        self.deregister_face_color(face_color_entity);
    }

    pub fn deregister_face_color(
        &mut self,
        face_color_entity: &Entity,
    ) {
        let Some(face_3d_entity) = self.color_to_face_map.remove(face_color_entity) else {
            panic!("face color entity not found");
        };

        self.face_to_color_map.remove(&face_3d_entity);
    }
}
