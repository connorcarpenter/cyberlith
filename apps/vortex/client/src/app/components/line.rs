use bevy_ecs::{entity::Entity, prelude::Component};

#[derive(Component)]
pub struct LineEntities {
    pub start: Entity,
    pub end_3d: Entity,
}

impl LineEntities {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end_3d: end }
    }
}