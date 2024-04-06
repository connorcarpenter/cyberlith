use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct Global {
    pub camera_entity: Entity,
}

impl Global {
    pub fn new(ui_camera_entity: Entity) -> Self {
        Self {
            camera_entity: ui_camera_entity,
        }
    }
}
