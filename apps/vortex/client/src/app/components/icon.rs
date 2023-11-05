use bevy_ecs::entity::Entity;

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
        for (connected_vertex, _) in self.connected_vertices {
            if *connected_vertex == old_entity {
                *connected_vertex = new_entity;
            }
        }
        for (connected_vertex_a, connected_vertex_b, _, _) in self.face_data {
            if *connected_vertex_a == old_entity {
                *connected_vertex_a = new_entity;
            }
            if *connected_vertex_b == old_entity {
                *connected_vertex_b = new_entity;
            }
        }
    }

    pub fn migrate_edge_entities(&mut self, old_2d_entity: Entity, new_2d_entity: Entity) {
        for (_, connected_edge_opt) in self.connected_vertices {
            if let Some(connected_edge) = connected_edge_opt {
                if *connected_edge == old_2d_entity {
                    *connected_edge = new_2d_entity;
                }
            }
        }
    }

    pub fn migrate_face_entities(&mut self, old_2d_entity: Entity, new_2d_entity: Entity) {
        for (_, _, face_2d_entity, _) in self.face_data {
            if *face_2d_entity == old_2d_entity {
                *face_2d_entity = new_2d_entity;
            }
        }
    }
}

fn migrate_vertex_trees(
    vertex_trees_opt: &mut Option<Vec<IconVertexEntry>>,
    old_2d_entity: Entity,
    new_2d_entity: Entity,
) {
    if let Some(vertex_trees) = vertex_trees_opt {
        for vertex_tree in vertex_trees {
            if vertex_tree.entity_2d == old_2d_entity {
                vertex_tree.entity_2d = new_2d_entity;
            }
            migrate_vertex_trees(
                &mut vertex_tree.children,
                old_2d_entity,
                new_2d_entity,
            );
        }
    }
}