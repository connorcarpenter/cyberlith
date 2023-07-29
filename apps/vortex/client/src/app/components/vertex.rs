use bevy_ecs::{entity::Entity, prelude::Component};

use math::Vec3;

// Just a marker, to distinguish from 3d version
#[derive(Component)]
pub struct Vertex2d;

impl Vertex2d {
    pub const RADIUS: f32 = 3.0;
    pub const SUBDIVISIONS: u16 = 12;
}

// for stored children vertexes undo/redo ...
#[derive(Clone)]
pub struct VertexEntry {
    pub entity: Entity,
    pub position: Vec3,
    pub children: Option<Vec<VertexEntry>>,
}

impl VertexEntry {
    pub fn new(entity: Entity, position: Vec3) -> Self {
        Self {
            entity,
            position,
            children: None,
        }
    }
}