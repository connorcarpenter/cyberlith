use bevy_ecs::prelude::Component;

// Just a marker for the select circle

#[derive(Component)]
pub struct SelectCircle;

impl SelectCircle {
    pub const RADIUS: f32 = 5.0;
}

// Just a marker for the select triangle

#[derive(Component)]
pub struct SelectTriangle;

impl SelectTriangle {
    pub const SIZE: f32 = 5.0;
}

#[derive(Component)]
pub struct SelectLine;
