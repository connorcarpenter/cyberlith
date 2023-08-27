use bevy_ecs::{entity::Entity, prelude::Component};

use render_api::base::Color;

// Just a marker, to distinguish from 3d version
#[derive(Component)]
pub struct FaceIcon2d {
    vertex_2d_a: Entity,
    vertex_2d_b: Entity,
    vertex_2d_c: Entity,
}

impl FaceIcon2d {
    pub const SIZE: f32 = 3.0;
    pub const HOVER_SIZE: f32 = FaceIcon2d::SIZE + 1.0;
    pub const DETECT_RADIUS: f32 = FaceIcon2d::SIZE + 1.0;
    pub const COLOR: Color = Color::GREEN;

    pub fn new(vertex_2d_a: Entity, vertex_2d_b: Entity, vertex_2d_c: Entity) -> Self {
        Self {
            vertex_2d_a,
            vertex_2d_b,
            vertex_2d_c,
        }
    }

    pub fn vertex_2d_a(&self) -> Entity {
        self.vertex_2d_a
    }

    pub fn vertex_2d_b(&self) -> Entity {
        self.vertex_2d_b
    }

    pub fn vertex_2d_c(&self) -> Entity {
        self.vertex_2d_c
    }
}
