use bevy_ecs::{entity::Entity, prelude::Component};

// LocalAnimRotation
#[derive(Component)]
pub struct LocalAnimRotation {
    pub frame_entity: Entity,
}

impl LocalAnimRotation {
    pub fn new(frame_entity: Entity) -> Self {
        Self {
            frame_entity,
        }
    }
}