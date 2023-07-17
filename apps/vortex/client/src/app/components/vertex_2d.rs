use bevy_ecs::prelude::Component;

// Just a marker, to distinguish from 3d version
#[derive(Component)]
pub struct Vertex2d;

impl Vertex2d {
    pub const RADIUS: f32 = 4.0;
    pub const SUBDIVISIONS: u16 = 12;
}