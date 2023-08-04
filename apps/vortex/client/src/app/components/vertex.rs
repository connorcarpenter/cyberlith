use bevy_ecs::{entity::Entity, prelude::Component};

use math::Vec3;
use render_api::base::Color;

// Just a marker, to distinguish from 3d version
#[derive(Component)]
pub struct Vertex2d;

impl Vertex2d {
    pub const RADIUS: f32 = 3.0;
    pub const SUBDIVISIONS: u16 = 12;
    pub const CHILD_COLOR: Color = Color::GREEN;
    pub const ROOT_COLOR: Color = Color::LIGHT_GREEN;
}

// for stored children vertexes undo/redo ...
#[derive(Clone)]
pub struct VertexEntry {
    pub entity_2d: Entity,
    pub entity_3d: Entity,
    pub position: Vec3,
    pub children: Option<Vec<VertexEntry>>,
}

impl VertexEntry {
    pub fn new(entity_2d: Entity, entity_3d: Entity, position: Vec3) -> Self {
        Self {
            entity_2d,
            entity_3d,
            position,
            children: None,
        }
    }
}

// for the editor compass
#[derive(Component)]
pub struct Compass;
