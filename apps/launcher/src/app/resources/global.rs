use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,
    pub camera_ui: Entity,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            camera_3d: Entity::PLACEHOLDER,
            camera_ui: Entity::PLACEHOLDER,
        }
    }
}
