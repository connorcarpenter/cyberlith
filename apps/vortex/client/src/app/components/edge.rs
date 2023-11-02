use bevy_ecs::{entity::Entity, prelude::Component};

#[derive(Component)]
pub struct Edge2dLocal {
    pub start: Entity,
    pub end: Entity,
}

impl Edge2dLocal {
    pub const NORMAL_THICKNESS: f32 = 1.0;
    pub const DETECT_THICKNESS: f32 = Self::NORMAL_THICKNESS + 1.0;
    pub const HOVER_THICKNESS: f32 = Self::NORMAL_THICKNESS + 1.0;
    pub const EDGE_ANGLE_LENGTH: f32 = 5.0;
    pub const EDGE_ANGLE_THICKNESS: f32 = 1.0;
    pub const EDGE_ANGLE_BASE_CIRCLE_RADIUS: f32 = ((Self::HOVER_THICKNESS + 1.0) / 2.0) * 1.2;
    pub const EDGE_ANGLE_END_CIRCLE_RADIUS: f32 = Self::EDGE_ANGLE_BASE_CIRCLE_RADIUS * 0.6;

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

// EdgeAngle
#[derive(Component)]
pub struct EdgeAngleLocal {
    radians: f32,
}

impl EdgeAngleLocal {
    pub fn new(radians: f32) -> Self {
        Self {
            radians,
        }
    }

    // angle in degrees
    pub fn get_radians(&self) -> f32 {
        self.radians
    }

    // angle in degrees
    pub fn set_radians(&mut self, radians: f32) {
        self.radians = radians;
    }
}