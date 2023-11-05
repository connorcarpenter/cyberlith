
use bevy_ecs::{entity::Entity, component::Component};

use math::Vec3;

// for stored children vertexes undo/redo ...
#[derive(Clone)]
pub struct IconVertexEntry {
    entity_2d: Entity,
    position: Vec3,
    children: Option<Vec<IconVertexEntry>>,
}

impl IconVertexEntry {
    pub fn new(entity_2d: Entity, position: Vec3) -> Self {
        Self {
            entity_2d,
            position,
            children: None,
        }
    }

    pub fn set_children(&mut self, children: Vec<IconVertexEntry>) {
        self.children = Some(children);
    }

    pub fn entity_2d(&self) -> Entity {
        self.entity_2d
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn children(&self) -> Option<Vec<IconVertexEntry>> {
        self.children.clone()
    }
}

// IconVertexData
#[derive(Clone)]
pub struct IconVertexActionData {
    pub(crate) connected_vertices: Vec<(Entity, Option<Entity>)>,
    pub(crate) face_data: Vec<(Entity, Entity, Entity, bool)>,
}

impl IconVertexActionData {

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