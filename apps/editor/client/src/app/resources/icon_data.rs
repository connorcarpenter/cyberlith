use std::collections::HashSet;

use bevy_ecs::entity::Entity;

pub struct IconVertexData {
    pub(crate) frame_entity_opt: Option<Entity>,
    pub(crate) edges: HashSet<Entity>,
    pub(crate) faces: HashSet<IconFaceKey>,
}

impl IconVertexData {
    pub fn new(frame_entity_opt: Option<Entity>) -> Self {
        Self {
            frame_entity_opt,
            edges: HashSet::new(),
            faces: HashSet::new(),
        }
    }

    pub fn add_edge(&mut self, edge_entity: Entity) {
        self.edges.insert(edge_entity);
    }

    pub fn remove_edge(&mut self, edge_entity: &Entity) {
        self.edges.remove(edge_entity);
    }

    pub fn add_face(&mut self, face_key: IconFaceKey) {
        self.faces.insert(face_key);
    }

    pub fn remove_face(&mut self, face_key: &IconFaceKey) {
        self.faces.remove(face_key);
    }
}

pub struct IconEdgeData {
    pub(crate) frame_entity_opt: Option<Entity>,
    pub(crate) vertex_entity_a: Entity,
    pub(crate) vertex_entity_b: Entity,
    pub(crate) faces: HashSet<IconFaceKey>,
}

impl IconEdgeData {
    pub fn new(
        frame_entity_opt: Option<Entity>,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
    ) -> Self {
        Self {
            frame_entity_opt,
            vertex_entity_a,
            vertex_entity_b,
            faces: HashSet::new(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct IconFaceKey {
    pub vertex_a: Entity,
    pub vertex_b: Entity,
    pub vertex_c: Entity,
}

impl IconFaceKey {
    pub fn new(vertex_a: Entity, vertex_b: Entity, vertex_c: Entity) -> Self {
        let mut vertices = vec![vertex_a, vertex_b, vertex_c];

        vertices.sort();

        Self {
            vertex_a: vertices[0],
            vertex_b: vertices[1],
            vertex_c: vertices[2],
        }
    }
}

pub struct IconFaceData {
    pub(crate) file_entity: Entity,
    pub(crate) frame_entity: Entity,

    pub(crate) local_entity: Entity,
    pub(crate) net_entity: Option<Entity>,

    edges: HashSet<Entity>,
}

impl IconFaceData {
    pub fn new(
        file_entity: Entity,
        frame_entity: Entity,
        local_entity: Entity,
    ) -> Self {
        Self {
            file_entity,
            frame_entity,
            local_entity,
            net_entity: None,
            edges: HashSet::new(),
        }
    }

    pub fn add_edge(&mut self, edge_entity: Entity) {
        self.edges.insert(edge_entity);
    }

    pub fn remove_edge(&mut self, edge_entity: &Entity) {
        self.edges.remove(edge_entity);
    }

    pub fn get_edges(&self) -> &HashSet<Entity> {
        &self.edges
    }
}
