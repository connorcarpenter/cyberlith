use bevy_ecs::{system::Resource, entity::Entity};

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,
    pub camera_ui: Entity,
}

impl Global {
    pub fn new(camera_3d: Entity, camera_ui: Entity) -> Self {
        Self {
            camera_3d,
            camera_ui,
        }
    }
}