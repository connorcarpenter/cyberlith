use bevy_ecs::prelude::Component;

// Just a marker for the select circle

#[derive(Component)]
pub struct SelectCircle;

impl SelectCircle {
    pub const RADIUS: f32 = 7.0;
}

#[derive(Component)]
pub struct SelectLine;
