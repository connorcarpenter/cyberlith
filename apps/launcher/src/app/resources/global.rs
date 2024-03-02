use bevy_ecs::{system::Resource, entity::Entity};

#[derive(Resource)]
pub struct Global {
    pub camera_entity: Entity,
}

impl Global {
    pub fn new(camera_entity: Entity) -> Self {
        Self {
            camera_entity,
        }
    }
}