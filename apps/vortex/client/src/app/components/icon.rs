
use bevy_ecs::{entity::Entity, component::Component};

// IconVertexData
#[derive(Clone)]
pub struct IconVertexActionData {
    pub(crate) connected_vertices: Vec<(Entity, Option<Entity>)>,
    pub(crate) face_data: Vec<(Entity, Entity, Entity, bool)>,
}

impl IconVertexActionData {

    pub fn new(
        connected_vertices: Vec<(Entity, Option<Entity>)>,
        face_data: Vec<(Entity, Entity, Entity, bool)>
    ) -> Self {
        Self {
            connected_vertices,
            face_data,
        }
    }

    pub fn migrate_vertex_entities(
        &mut self,
        old_entity: Entity,
        new_entity: Entity,
    ) {
        for (connected_vertex, _) in &mut self.connected_vertices {
            if *connected_vertex == old_entity {
                *connected_vertex = new_entity;
            }
        }
        for (connected_vertex_a, connected_vertex_b, _, _) in &mut self.face_data {
            if *connected_vertex_a == old_entity {
                *connected_vertex_a = new_entity;
            }
            if *connected_vertex_b == old_entity {
                *connected_vertex_b = new_entity;
            }
        }
    }

    pub fn migrate_edge_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for (_, connected_edge_opt) in &mut self.connected_vertices {
            if let Some(connected_edge) = connected_edge_opt {
                if *connected_edge == old_entity {
                    *connected_edge = new_entity;
                }
            }
        }
    }

    pub fn migrate_face_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for (_, _, face_entity, _) in &mut self.face_data {
            if *face_entity == old_entity {
                *face_entity = new_entity;
            }
        }
    }
}

// Edges

#[derive(Component)]
pub struct IconEdgeLocal {
    pub start: Entity,
    pub end: Entity,
}

impl IconEdgeLocal {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self { start, end }
    }
}

// Just a marker
#[derive(Component)]
pub struct IconLocalFace {
    // DON'T REMOVE THESE JUST YET! WE WILL NEED THEM WHEN SYNCING
    vertex_a: Entity,
    vertex_b: Entity,
    vertex_c: Entity,
}

impl IconLocalFace {

    pub fn new(vertex_a: Entity, vertex_b: Entity, vertex_c: Entity) -> Self {
        Self {
            vertex_a,
            vertex_b,
            vertex_c,
        }
    }

    pub fn vertex_a(&self) -> Entity {
        self.vertex_a
    }

    pub fn vertex_b(&self) -> Entity {
        self.vertex_b
    }

    pub fn vertex_c(&self) -> Entity {
        self.vertex_c
    }
}