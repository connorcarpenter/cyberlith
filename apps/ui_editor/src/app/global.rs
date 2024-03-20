use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct Global {
    pub camera_ui: Entity,
}

impl Global {
    pub fn new(camera_ui: Entity) -> Self {
        Self { camera_ui }
    }
}
