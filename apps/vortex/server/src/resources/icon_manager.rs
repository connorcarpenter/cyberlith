use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_server::{CommandsExt, Server};

pub struct IconVertexData {
    edges: HashSet<Entity>,
    // faces
    faces: HashSet<Entity>,
}

impl IconVertexData {
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

pub struct IconEdgeData {
    vertex_a: Entity,
    vertex_b: Entity,
}

impl IconEdgeData {
    pub fn new(start: Entity, end: Entity) -> Self {
        Self {
            vertex_a: start,
            vertex_b: end,
        }
    }
}

pub struct IconFaceData {
    file_entity: Entity,
    face_index: usize,
    vertex_a: Entity,
    vertex_b: Entity,
    vertex_c: Entity,
}

impl IconFaceData {
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
pub struct IconManager {
    // vertex entity -> vertex data
    vertices: HashMap<Entity, IconVertexData>,
    // edge entity -> connected vertex entities
    edges: HashMap<Entity, IconEdgeData>,
    // face entity -> connected vertices
    faces: HashMap<Entity, IconFaceData>,
    // file entity -> face entity list
    file_face_indices: HashMap<Entity, Vec<Entity>>,
}

impl Default for IconManager {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            faces: HashMap::new(),
            file_face_indices: HashMap::new(),
        }
    }
}

impl IconManager {
    pub fn has_vertex(&self, entity: &Entity) -> bool {
        self.vertices.contains_key(entity)
    }

    pub fn has_edge(&self, entity: &Entity) -> bool {
        self.edges.contains_key(entity)
    }

    pub fn has_face(&self, entity: &Entity) -> bool {
        self.faces.contains_key(entity)
    }

    pub fn get_face_index(&self, entity: &Entity) -> Option<usize> {
        if let Some(face_data) = self.faces.get(entity) {
            Some(face_data.face_index)
        } else {
            None
        }
    }

    pub fn face_entity_from_index(
        &self,
        file_entity: &Entity,
        face_index: usize,
    ) -> Option<Entity> {
        if let Some(file_face_indices) = self.file_face_indices.get(file_entity) {
            Some(file_face_indices[face_index])
        } else {
            None
        }
    }

    pub fn on_create_vertex(&mut self, vertex_entity: Entity) {
        // info!("on_create_vertex: {:?} {:?}", entity, parent_opt);

        info!("inserting icon vert entity: `{:?}`", vertex_entity,);

        self.vertices.insert(vertex_entity, IconVertexData::new());
    }

    pub fn on_create_edge(
        &mut self,
        start_vertex_entity: Entity,
        edge_entity: Entity,
        end_vertex_entity: Entity,
    ) {
        self.edges.insert(
            edge_entity,
            IconEdgeData::new(start_vertex_entity, end_vertex_entity),
        );

        for vertex_entity in [start_vertex_entity, end_vertex_entity] {
            let Some(vertex_data) = self.vertices.get_mut(&vertex_entity) else {
                panic!("on_create_icon_edge: vertex entity `{:?}` not found!", vertex_entity);
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
            IconFaceData::new(*file_entity, face_index, vertex_a, vertex_b, vertex_c),
        );

        // add faces to vertices
        for vertex_entity in [vertex_a, vertex_b, vertex_c] {
            let Some(data) = self.vertices.get_mut(&vertex_entity) else {
                panic!("on_create_face: vertex entity `{:?}` not found!", vertex_entity);
            };
            data.add_face(face_entity);
        }

        // TODO: add face to edges
    }

    fn assign_index_to_new_face(
        &mut self,
        file_entity: &Entity,
        old_index_opt: Option<usize>,
        face_3d_entity: &Entity,
    ) -> usize {
        info!(
            "assign_index_to_new_face(entity: `{:?}`, index: {:?})",
            face_3d_entity, old_index_opt
        );
        if !self.file_face_indices.contains_key(file_entity) {
            self.file_face_indices.insert(*file_entity, Vec::new());
        }
        let file_face_indices = self.file_face_indices.get_mut(file_entity).unwrap();

        let new_index = file_face_indices.len();

        if let Some(old_index) = old_index_opt {
            if new_index != old_index {
                panic!(
                    "something went wrong, got new index `{:?}` but old index was `{:?}`",
                    new_index, old_index
                );
            }
        }

        file_face_indices.push(*face_3d_entity);

        new_index
    }

    pub fn deregister_vertex(&mut self, vertex_entity: &Entity) -> Option<IconVertexData> {
        self.vertices.remove(vertex_entity)
    }

    pub fn deregister_edge(&mut self, edge_entity: &Entity) -> Option<IconEdgeData> {
        self.edges.remove(edge_entity)
    }

    pub fn deregister_face(&mut self, face_entity: &Entity) -> Option<IconFaceData> {
        let Some(face_data) = self.faces.remove(face_entity) else {
            return None;
        };

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

        Some(face_data)
    }

    pub fn on_client_despawn_vertex(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        vertex_entity: &Entity,
    ) -> Vec<Entity> {
        let mut entities_to_despawn = Vec::new();

        let vertex_data = self.vertices.remove(vertex_entity).unwrap();

        for edge_entity in vertex_data.edges.iter() {
            entities_to_despawn.push(*edge_entity);

            self.on_client_despawn_edge(edge_entity);
        }

        for face_entity in vertex_data.faces.iter() {
            entities_to_despawn.push(*face_entity);

            self.on_client_despawn_face(face_entity);
        }

        info!(
            "on_client_despawn_vertex: entity `{:?}`, entities_to_despawn: `{:?}`",
            vertex_entity, entities_to_despawn,
        );

        for child_entity in entities_to_despawn.iter() {
            commands
                .entity(*child_entity)
                .take_authority(server)
                .despawn();
        }

        entities_to_despawn
    }

    pub fn on_client_despawn_edge(&mut self, edge_entity: &Entity) {
        let Some(edge_data) = self.deregister_edge(edge_entity) else {
            warn!("edge entity `{:?}` not found, perhaps was already despawned?", edge_entity);
            return;
        };

        for vertex_entity in [edge_data.vertex_a, edge_data.vertex_b] {
            if let Some(data) = self.vertices.get_mut(&vertex_entity) {
                info!(
                    "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
                    vertex_entity, edge_entity
                );
                data.remove_edge(edge_entity);
            }
        }
    }

    pub(crate) fn on_client_despawn_face(&mut self, face_entity: &Entity) {
        let Some(face_data) = self.deregister_face(face_entity) else {
            warn!("face entity `{:?}` not found, perhaps was already despawned?", face_entity);
            return;
        };

        // remove face from vertex data
        for vertex_entity in [face_data.vertex_a, face_data.vertex_b, face_data.vertex_c] {
            if let Some(data) = self.vertices.get_mut(&vertex_entity) {
                info!(
                    "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
                    vertex_entity, face_entity
                );
                data.remove_face(face_entity);
            }
        }
    }
}