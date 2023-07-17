use bevy_ecs::prelude::Component;

// Just a marker for the hover circle

#[derive(Component)]
pub struct HoverCircle;

impl HoverCircle {
    pub const RADIUS: f32 = 11.0;
}