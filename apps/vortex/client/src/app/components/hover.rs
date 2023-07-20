use bevy_ecs::prelude::Component;

use crate::app::components::Vertex2d;

// Just a marker for the hover circle

#[derive(Component)]
pub struct HoverCircle;

impl HoverCircle {
    pub const DISPLAY_RADIUS: f32 = 8.0;
    pub const DETECT_RADIUS: f32 = Vertex2d::RADIUS + 1.0;
}
