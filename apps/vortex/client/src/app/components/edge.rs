use bevy_ecs::{entity::Entity, prelude::Component};

#[derive(Component)]
pub struct Edge2dLocal {
    pub start: Entity,
    pub end: Entity,
}

impl Edge2dLocal {
    pub const NORMAL_THICKNESS: f32 = 1.0;
    pub const DETECT_THICKNESS: f32 = Edge2dLocal::NORMAL_THICKNESS + 1.0;
    pub const HOVER_THICKNESS: f32 = Edge2dLocal::NORMAL_THICKNESS + 1.0;

    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end }
    }
}

#[derive(Component)]
pub struct Edge3dLocal {
    pub start: Entity,
    pub end: Entity,
}

impl Edge3dLocal {
    pub const NORMAL_THICKNESS: f32 = 1.0;

    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end }
    }
}

#[derive(Component)]
pub struct EdgeAngleLocal {
    pub angle: f32,
}

impl EdgeAngleLocal {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }
}