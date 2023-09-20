use std::collections::HashSet;

use bevy_ecs::entity::Entity;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CanvasShape {
    RootVertex,
    Vertex,
    Edge,
    Face,
}

pub struct Vertex3dData {
    pub(crate) entity_2d: Entity,
    pub(crate) edges_3d: HashSet<Entity>,
    pub(crate) faces_3d: HashSet<FaceKey>,
    pub(crate) owner_opt: Option<Entity>,
    pub(crate) parent_3d_entity_opt: Option<Entity>,
    pub(crate) children_3d_entities_opt: Option<HashSet<Entity>>,
}

impl Vertex3dData {
    pub fn new(
        parent_3d_entity_opt: Option<Entity>,
        entity_2d: Entity,
        owner_opt: Option<Entity>,
    ) -> Self {
        Self {
            parent_3d_entity_opt,
            entity_2d,
            edges_3d: HashSet::new(),
            faces_3d: HashSet::new(),
            owner_opt,
            children_3d_entities_opt: None,
        }
    }

    pub fn add_edge(&mut self, edge_3d_entity: Entity) {
        self.edges_3d.insert(edge_3d_entity);
    }

    pub fn remove_edge(&mut self, edge_3d_entity: &Entity) {
        self.edges_3d.remove(edge_3d_entity);
    }

    pub fn add_face(&mut self, face_key: FaceKey) {
        self.faces_3d.insert(face_key);
    }

    pub fn remove_face(&mut self, face_key: &FaceKey) {
        self.faces_3d.remove(face_key);
    }

    pub fn add_child(&mut self, child_3d_entity: Entity) {
        if let Some(children_3d_entities) = &mut self.children_3d_entities_opt {
            children_3d_entities.insert(child_3d_entity);
        } else {
            let mut children_3d_entities = HashSet::new();
            children_3d_entities.insert(child_3d_entity);
            self.children_3d_entities_opt = Some(children_3d_entities);
        }
    }

    pub fn remove_child(&mut self, child_3d_entity: &Entity) {
        let mut remove_all = false;
        if let Some(children_3d_entities) = &mut self.children_3d_entities_opt {
            children_3d_entities.remove(child_3d_entity);
            if children_3d_entities.is_empty() {
                remove_all = true;
            }
        }
        if remove_all {
            self.children_3d_entities_opt = None;
        }
    }
}

pub struct Edge3dData {
    pub(crate) entity_2d: Entity,
    pub(crate) vertex_a_3d_entity: Entity,
    pub(crate) vertex_b_3d_entity: Entity,
    pub(crate) faces_3d: HashSet<FaceKey>,
    // base circle, line, end circle
    pub(crate) angle_entities_opt: Option<(Entity, Entity, Entity)>,
}

impl Edge3dData {
    pub fn new(
        entity_2d: Entity,
        vertex_a_3d_entity: Entity,
        vertex_b_3d_entity: Entity,
        angle_entities_opt: Option<(Entity, Entity, Entity)>,
    ) -> Self {
        Self {
            entity_2d,
            vertex_a_3d_entity,
            vertex_b_3d_entity,
            faces_3d: HashSet::new(),
            angle_entities_opt,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct FaceKey {
    pub vertex_3d_a: Entity,
    pub vertex_3d_b: Entity,
    pub vertex_3d_c: Entity,
}

impl FaceKey {
    pub fn new(vertex_a: Entity, vertex_b: Entity, vertex_c: Entity) -> Self {
        let mut vertices = vec![vertex_a, vertex_b, vertex_c];

        vertices.sort();

        Self {
            vertex_3d_a: vertices[0],
            vertex_3d_b: vertices[1],
            vertex_3d_c: vertices[2],
        }
    }
}

pub struct FaceData {
    pub(crate) entity_3d: Option<Entity>,
    pub(crate) entity_2d: Entity,
    pub(crate) file_entity: Entity,

    pub(crate) edge_3d_a: Entity,
    pub(crate) edge_3d_b: Entity,
    pub(crate) edge_3d_c: Entity,
}

impl FaceData {
    pub fn new(
        entity_2d: Entity,
        file_entity: Entity,
        edge_3d_a: Entity,
        edge_3d_b: Entity,
        edge_3d_c: Entity,
    ) -> Self {
        Self {
            entity_2d,
            entity_3d: None,
            file_entity,
            edge_3d_a,
            edge_3d_b,
            edge_3d_c,
        }
    }
}
