use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct Global {
    pub scene_camera_entity: Entity,
}

impl Global {
    pub fn new(scene_camera_entity: Entity) -> Self {
        Self {
            scene_camera_entity,
        }
    }
}
