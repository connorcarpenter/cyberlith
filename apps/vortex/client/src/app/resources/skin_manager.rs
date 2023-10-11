use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct SkinManager {
    face_colors: HashMap<Entity, Entity>,
}

impl Default for SkinManager {
    fn default() -> Self {
        Self {
            face_colors: HashMap::new(),
        }
    }
}

impl SkinManager {
    pub(crate) fn get_face_color(&self, face_3d_entity: Entity) -> Option<&Entity> {
        self.face_colors.get(&face_3d_entity)
    }
}