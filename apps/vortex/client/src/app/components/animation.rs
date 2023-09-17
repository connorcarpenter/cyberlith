use bevy_ecs::{entity::Entity, prelude::Component};

// LocalAnimRotation
#[derive(Component)]
pub struct LocalAnimRotation {
    pub frame_entity: Entity,
    pub vertex_3d_entity: Entity,
}

impl LocalAnimRotation {
    pub fn new(frame_entity: Entity, vertex_3d_entity: Entity) -> Self {
        Self {
            frame_entity,
            vertex_3d_entity,
        }
    }
}