use bevy_ecs::{entity::Entity, prelude::Component};

#[derive(Component)]
pub struct Edge2d {
    pub start: Entity,
    pub end_3d: Entity,
}

impl Edge2d {
    pub const HOVER_THICKNESS: f32 = 2.0;
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end_3d: end }
    }
}

#[derive(Component)]
pub struct Edge3d {
    pub start: Entity,
    pub end: Entity,
}

impl Edge3d {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end }
    }
}