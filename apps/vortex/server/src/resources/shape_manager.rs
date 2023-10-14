use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_server::{CommandsExt, Server};

pub enum VertexData {
    Skel(SkelVertexData),
    Mesh(MeshVertexData),
}

pub struct SkelVertexData {
    // parent entity, edge entity
    parent_and_edge_opt: Option<(Entity, Entity)>,
    // children map from vertex entity to edge entity
    children: Option<HashMap<Entity, Entity>>,
}

impl SkelVertexData {
    fn new(parent_and_edge_opt: Option<(Entity, Entity)>) -> Self {
        Self {
            parent_and_edge_opt,
            children: None,
        }
    }

    fn add_child(&mut self, vertex_entity: Entity, edge_entity: Entity) {
        self.children
            .get_or_insert_with(|| HashMap::new())
            .insert(vertex_entity, edge_entity);
    }

    fn remove_child(&mut self, entity: &Entity) -> Option<Entity> {
        if let Some(children) = self.children.as_mut() {
            return children.remove(&entity);
        }
        return None;
    }
}

pub struct MeshVertexData {
    edges: HashSet<Entity>,
    // faces
    faces: HashSet<Entity>,
}

impl MeshVertexData {
    fn new() -> Self {
        Self {
            edges: HashSet::new(),
            faces: HashSet::new(),
        }
    }

    fn add_edge(&mut self, edge_entity: Entity) {
        self.edges.insert(edge_entity);
    }

    // returns edge entity
    fn remove_edge(&mut self, edge_entity: &Entity) {
        self.edges.remove(edge_entity);
    }

    fn add_face(&mut self, face_entity: Entity) {
        self.faces.insert(face_entity);
    }

    fn remove_face(&mut self, face_entity: &Entity) {
        self.faces.remove(&face_entity);
    }
}

pub struct EdgeData {
    vertex_a: Entity,
    vertex_b: Entity,
}

impl EdgeData {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self {
            vertex_a: start,
            vertex_b: end,
        }
    }
}

struct FaceData {
    file_entity: Entity,
    face_index: usize,
    vertex_a: Entity,
    vertex_b: Entity,
    vertex_c: Entity,
}

impl FaceData {
    pub fn new(
        file_entity: Entity,
        face_index: usize,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
    ) -> Self {
        Self {
            file_entity,
            face_index,
            vertex_a,
            vertex_b,
            vertex_c,
        }
    }
}

#[derive(Resource)]
pub struct ShapeManager {
    // vertex entity -> vertex data
    vertices: HashMap<Entity, VertexData>,
    // edge entity -> connected vertex entities
    edges: HashMap<Entity, EdgeData>,
    // face entity -> connected vertices
    faces: HashMap<Entity, FaceData>,
    // file entity -> face entity list
    file_face_indices: HashMap<Entity, Vec<Entity>>,
}

impl Default for ShapeManager {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            faces: HashMap::new(),
            file_face_indices: HashMap::new(),
        }
    }
}

impl ShapeManager {
    pub fn has_vertex(&self, entity: &Entity) -> bool {
        self.vertices.contains_key(entity)
    }

    pub fn has_edge(&self, entity: &Entity) -> bool {
        self.edges.contains_key(entity)
    }

    pub fn has_face(&self, entity: &Entity) -> bool {
        self.faces.contains_key(entity)
    }

    pub fn get_vertex_parent_and_edge(&self, entity: &Entity) -> Option<(Entity, Entity)> {
        if let Some(vertex_data) = self.vertices.get(entity) {
            match vertex_data {
                VertexData::Skel(skel_data) => skel_data.parent_and_edge_opt,
                VertexData::Mesh(_) => {
                    panic!("should not call this on a mesh vertex!");
                }
            }
        } else {
            None
        }
    }

    pub fn get_face_index(&self, entity: &Entity) -> Option<usize> {
        if let Some(face_data) = self.faces.get(entity) {
            Some(face_data.face_index)
        } else {
            None
        }
    }

    pub fn face_entity_from_index(&self, file_entity: &Entity, face_index: usize) -> Option<Entity> {
        if let Some(file_face_indices) = self.file_face_indices.get(file_entity) {
            Some(file_face_indices[face_index])
        } else {
            None
        }
    }

    pub fn on_create_mesh_vertex(&mut self, vertex_entity: Entity) {
        // info!("on_create_mesh_vertex: {:?} {:?}", entity, parent_opt);

        info!("inserting mesh vert entity: `{:?}`", vertex_entity,);

        self.vertices
            .insert(vertex_entity, VertexData::Mesh(MeshVertexData::new()));
    }

    pub fn on_create_skel_vertex(
        &mut self,
        vertex_entity: Entity,
        edge_and_parent_opt: Option<(Entity, Entity)>,
    ) {
        // info!("on_create_skel_vertex: {:?} {:?}", entity, parent_opt);

        if let Some((_, parent_entity)) = edge_and_parent_opt {
            if !self.vertices.contains_key(&parent_entity) {
                panic!("on_create_skel_vertex: parent entity `{:?}` not found! Vertex Waitlist should handle this...", parent_entity);
            }
        }

        info!(
            "inserting skel vertex entity: `{:?}`, edge_and_parent is `{:?}`",
            vertex_entity, edge_and_parent_opt
        );

        if let Some((edge_entity, parent_entity)) = edge_and_parent_opt {
            self.vertices.insert(
                vertex_entity,
                VertexData::Skel(SkelVertexData::new(Some((parent_entity, edge_entity)))),
            );
            let Some(VertexData::Skel(parent_value)) = self.vertices.get_mut(&parent_entity) else {
                panic!("shouldn't be able to happen!");
            };
            parent_value.add_child(vertex_entity, edge_entity);
        } else {
            self.vertices
                .insert(vertex_entity, VertexData::Skel(SkelVertexData::new(None)));
        }
    }

    pub fn on_create_mesh_edge(
        &mut self,
        start_vertex_entity: Entity,
        edge_entity: Entity,
        end_vertex_entity: Entity,
    ) {
        self.edges.insert(
            edge_entity,
            EdgeData::new(start_vertex_entity, end_vertex_entity),
        );

        for vertex_entity in [start_vertex_entity, end_vertex_entity] {
            let Some(VertexData::Mesh(vertex_data)) = self.vertices.get_mut(&vertex_entity) else {
                panic!("on_create_mesh_edge: vertex entity `{:?}` not found!", vertex_entity);
            };
            vertex_data.add_edge(edge_entity);
        }
    }

    pub fn on_create_face(
        &mut self,
        file_entity: &Entity,
        old_index_opt: Option<usize>,
        face_entity: Entity,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
    ) {
        // assign index
        let face_index = self.assign_index_to_new_face(file_entity, old_index_opt, &face_entity);

        self.faces.insert(
            face_entity,
            FaceData::new(*file_entity, face_index, vertex_a, vertex_b, vertex_c),
        );

        // add faces to vertices
        for vertex_entity in [vertex_a, vertex_b, vertex_c] {
            let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&vertex_entity) else {
                panic!("on_create_face: vertex entity `{:?}` not found!", vertex_entity);
            };
            data.add_face(face_entity);
        }

        // TODO: add face to edges


    }

    fn assign_index_to_new_face(&mut self, file_entity: &Entity, old_index_opt: Option<usize>, face_3d_entity: &Entity) -> usize {
        info!("assign_index_to_new_face(entity: `{:?}`, index: {:?})", face_3d_entity, old_index_opt);
        if !self.file_face_indices.contains_key(file_entity) {
            self.file_face_indices.insert(*file_entity, Vec::new());
        }
        let file_face_indices = self.file_face_indices.get_mut(file_entity).unwrap();

        let new_index = file_face_indices.len();

        if let Some(old_index) = old_index_opt {
            if new_index != old_index {
                panic!("something went wrong, got new index `{:?}` but old index was `{:?}`", new_index, old_index);
            }
        }

        file_face_indices.push(*face_3d_entity);

        new_index
    }

    pub fn deregister_vertex(&mut self, vertex_entity: &Entity) -> Option<VertexData> {
        self.vertices.remove(vertex_entity)
    }

    pub fn deregister_edge(&mut self, edge_entity: &Entity) -> Option<EdgeData> {
        self.edges.remove(edge_entity)
    }

    pub fn on_client_despawn_vertex(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        entity: &Entity,
    ) -> Vec<Entity> {
        let entities_to_despawn = match self.vertices.get(entity) {
            Some(VertexData::Skel(_)) => self.on_client_despawn_skel_vertex(entity),
            Some(VertexData::Mesh(_)) => self.on_client_despawn_mesh_vertex(entity),
            None => {
                panic!(
                    "on_client_despawn_vertex: vertex entity `{:?}` not found!",
                    entity
                );
            }
        };

        info!(
            "on_client_despawn_vertex: entity `{:?}`, entities_to_despawn: `{:?}`",
            entity, entities_to_despawn,
        );

        for child_entity in entities_to_despawn.iter() {
            commands
                .entity(*child_entity)
                .take_authority(server)
                .despawn();
        }

        entities_to_despawn
    }

    fn on_client_despawn_skel_vertex(&mut self, vertex_entity: &Entity) -> Vec<Entity> {
        let mut entities_to_despawn = Vec::new();

        // remove entry
        let removed_entry =
            self.remove_skel_vertex_and_collect_children(vertex_entity, &mut entities_to_despawn);

        // remove entry from parent's children
        if let Some((parent_entity, _)) = removed_entry.parent_and_edge_opt {
            if let Some(VertexData::Skel(parent)) = self.vertices.get_mut(&parent_entity) {
                if let Some(edge_entity) = parent.remove_child(vertex_entity) {
                    entities_to_despawn.push(edge_entity);
                }
            }
        }

        return entities_to_despawn;
    }

    fn on_client_despawn_mesh_vertex(&mut self, vertex_entity: &Entity) -> Vec<Entity> {
        let mut entities_to_despawn = Vec::new();

        let VertexData::Mesh(vertex_data) = self.vertices.remove(vertex_entity).unwrap() else {
            panic!("shouldn't be able to happen!");
        };

        for edge_entity in vertex_data.edges.iter() {
            entities_to_despawn.push(*edge_entity);

            self.on_client_despawn_edge(edge_entity);
        }

        for face_entity in vertex_data.faces.iter() {
            entities_to_despawn.push(*face_entity);

            self.on_client_despawn_face(face_entity);
        }

        entities_to_despawn
    }

    pub fn on_client_despawn_edge(&mut self, edge_entity: &Entity) {
        let Some(edge_data) = self.deregister_edge(edge_entity) else {
            warn!("edge entity `{:?}` not found, perhaps was already despawned?", edge_entity);
            return;
        };

        for vertex_entity in [edge_data.vertex_a, edge_data.vertex_b] {
            if let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&vertex_entity) {
                info!(
                    "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
                    vertex_entity, edge_entity
                );
                data.remove_edge(edge_entity);
            }
        }
    }

    pub(crate) fn on_client_despawn_face(&mut self, face_entity: &Entity) {
        let face_data = self.faces.remove(face_entity).unwrap();

        // remove face from vertex data
        for vertex_entity in [face_data.vertex_a, face_data.vertex_b, face_data.vertex_c] {
            if let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&vertex_entity) {
                info!(
                    "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
                    vertex_entity, face_entity
                );
                data.remove_face(face_entity);
            }
        }

        // remove face from file face list
        let file_entity = face_data.file_entity;
        let face_index = face_data.face_index;
        let file_face_indices = self.file_face_indices.get_mut(&file_entity).unwrap();
        file_face_indices.remove(face_index);
        for i in face_index..file_face_indices.len() {
            let face_entity = file_face_indices[i];
            let face_data = self.faces.get_mut(&face_entity).unwrap();
            face_data.face_index = i;
        }
    }

    fn remove_skel_vertex_and_collect_children(
        &mut self,
        entity: &Entity,
        entities_to_despawn: &mut Vec<Entity>,
    ) -> SkelVertexData {
        let VertexData::Skel(removed_entry) = self.deregister_vertex(entity).unwrap() else {
            panic!("shouldn't be able to happen!");
        };

        // handle children
        if let Some(removed_entry_children) = &removed_entry.children {
            for (child_entity, edge_entity) in removed_entry_children {
                self.remove_skel_vertex_and_collect_children(&child_entity, entities_to_despawn);
                entities_to_despawn.push(*child_entity);
                entities_to_despawn.push(*edge_entity);
            }
        }

        removed_entry
    }
}
