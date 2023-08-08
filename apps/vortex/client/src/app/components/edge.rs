use bevy_ecs::{entity::Entity, prelude::Component};

#[derive(Component)]
pub struct Edge2dLocal {
    pub start: Entity,
    pub end: Entity,
}

impl Edge2dLocal {
    pub const HOVER_THICKNESS: f32 = 2.0;
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end }
    }
}