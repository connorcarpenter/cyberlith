use bevy_ecs::{entity::Entity, system::Resource};

#[derive(Resource)]
pub struct Global {
    pub ui_camera_entity: Entity,
    pub active_ui_entity: Entity,
}

impl Global {
    pub fn new(ui_camera_entity: Entity, active_ui_entity: Entity) -> Self {
        Self {
            ui_camera_entity,
            active_ui_entity,
        }
    }
}
