use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_server::{CommandsExt, Server};

use editor_proto::components::IconFrame;

pub struct IconVertexData {
    frame_entity: Entity,
    edges: HashSet<Entity>,
    // faces
    faces: HashSet<Entity>,
}

impl IconVertexData {
    fn new(frame_entity: Entity) -> Self {
        Self {
            frame_entity,
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
    frame_entity: Entity,
    vertex_a: Entity,
    vertex_b: Entity,
}

impl IconEdgeData {
    pub fn new(frame_entity: Entity, start: Entity, end: Entity) -> Self {
        Self {
            frame_entity,
            vertex_a: start,
            vertex_b: end,
        }
    }
}

pub struct IconFaceData {
    frame_entity: Entity,
    face_index: usize,
    vertex_a: Entity,
    vertex_b: Entity,
    vertex_c: Entity,
}

impl IconFaceData {
    pub fn new(
        frame_entity: Entity,
        face_index: usize,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
    ) -> Self {
        Self {
            frame_entity,
            face_index,
            vertex_a,
            vertex_b,
            vertex_c,
        }
    }
}

pub struct FileFrameData {
    frames: HashMap<Entity, FrameData>,
    frame_list: Vec<Option<Entity>>,
}

impl FileFrameData {
    fn new() -> Self {
        Self {
            frames: HashMap::new(),
            frame_list: Vec::new(),
        }
    }

    fn add_frame(
        &mut self,
        frame_entity: Entity,
        frame_order: usize,
        mut frame_q_opt: Option<&mut Query<&mut IconFrame>>,
    ) {
        info!("--- add frame ---");
        for i in 0..self.frame_list.len() {
            info!("index: {}, entity: {:?}", i, self.frame_list[i]);
        }
        info!("- op -");

        self.frames.insert(frame_entity, FrameData::new(frame_order as u8));

        // add to frame_list
        if frame_order >= self.frame_list.len() {
            self.frame_list.resize(frame_order + 1, None);
            // set frame entity
            self.frame_list[frame_order] = Some(frame_entity);
        } else {
            info!(
                "add_frame: index: {:?}, entity: `{:?}`",
                frame_order, frame_entity
            );
            self.frame_list.insert(frame_order, Some(frame_entity));

            // move all elements after frame_order up one
            for i in frame_order + 1..self.frame_list.len() {
                // update frame_order in AnimFrame using frame_q_opt
                if let Some(frame_q) = frame_q_opt.as_mut() {
                    let entity = self.frame_list[i].unwrap();
                    let Ok(mut frame) = frame_q.get_mut(entity) else {
                        panic!("frame not found");
                    };
                    frame.set_order(i as u8);
                    let Some(frame_data) = self.frames.get_mut(&entity) else {
                        panic!("frame data not found");
                    };
                    frame_data.order = i as u8;
                }
            }
        }

        info!("--- result ---");
        for i in 0..self.frame_list.len() {
            info!("index: {}, entity: {:?}", i, self.frame_list[i]);
        }
    }

    fn remove_frame(
        &mut self,
        frame_entity: &Entity,
        frame_q_opt: Option<&mut Query<&mut IconFrame>>,
    ) -> Option<FrameData> {
        let Some(frame_data) = self.frames.remove(frame_entity) else {
            panic!("frame data not found");
        };

        let frame_order = {
            let mut frame_order_opt = None;
            for (frame_index, frame_item) in self.frame_list.iter().enumerate() {
                if let Some(frame_item) = frame_item {
                    if frame_item == frame_entity {
                        frame_order_opt = Some(frame_index);
                        break;
                    }
                }
            }
            frame_order_opt.unwrap()
        };

        // get frame_order of frame_entity
        if let Some(frame_q) = frame_q_opt {
            // move all elements after frame_order down one
            for i in frame_order..self.frame_list.len() - 1 {
                self.frame_list[i] = self.frame_list[i + 1];

                // update frame_order in IconFrame using frame_q_opt
                let entity = self.frame_list[i].unwrap();
                if let Ok(mut frame) = frame_q.get_mut(entity) {
                    frame.set_order(i as u8);
                }
                let Some(frame_data) = self.frames.get_mut(&entity) else {
                    panic!("frame data not found");
                };
                frame_data.order = i as u8;
            }

            self.frame_list.truncate(self.frame_list.len() - 1);
        }

        Some(frame_data)
    }

    pub(crate) fn update_frame_order(&mut self, new_frame_entity: &Entity, new_order: u8, frame_q: &mut Query<&mut IconFrame>) {
        let Some(new_frame_data) = self.frames.get_mut(new_frame_entity) else {
            panic!("frame entity not found");
        };
        let old_order = new_frame_data.order;
        new_frame_data.order = new_order;

        // get entity at new order index
        let old_frame_entity = self.frame_list[new_order as usize].unwrap();
        let Some(old_frame_data) = self.frames.get_mut(&old_frame_entity) else {
            panic!("frame entity not found");
        };
        old_frame_data.order = old_order;

        // now swap the entities in the frame list, new_order and old_order
        self.frame_list[new_order as usize] = Some(*new_frame_entity);
        self.frame_list[old_order as usize] = Some(old_frame_entity);

        // new frame entity is already updated, but need to update old frame entity
        let Ok(mut old_frame) = frame_q.get_mut(old_frame_entity) else {
            panic!("frame not found");
        };
        old_frame.set_order(old_order);
    }

    fn add_vertex(&mut self, frame_entity: Entity, shape_entity: Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.add_vertex(shape_entity);
    }

    fn remove_vertex(&mut self, frame_entity: &Entity, shape_entity: &Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.remove_vertex(shape_entity);
    }

    fn add_edge(&mut self, frame_entity: Entity, shape_entity: Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.add_edge(shape_entity);
    }

    fn remove_edge(&mut self, frame_entity: &Entity, edge_entity: &Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.remove_edge(edge_entity);
    }

    fn add_face(&mut self, frame_entity: Entity, face_entity: Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.add_face(face_entity);
    }

    fn remove_face(&mut self, frame_entity: &Entity, shape_entity: &Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.remove_face(shape_entity);
    }
}

pub struct FrameData {
    order: u8,
    vertices: HashSet<Entity>,
    edges: HashSet<Entity>,
    faces: HashSet<Entity>,
}

impl FrameData {
    fn new(order: u8) -> Self {
        Self {
            order,
            vertices: HashSet::new(),
            edges: HashSet::new(),
            faces: HashSet::new(),
        }
    }

    fn add_vertex(&mut self, entity: Entity) {
        self.vertices.insert(entity);
    }

    fn remove_vertex(&mut self, entity: &Entity) {
        self.vertices.remove(entity);
    }

    fn add_edge(&mut self, entity: Entity) {
        self.edges.insert(entity);
    }

    fn remove_edge(&mut self, entity: &Entity) {
        self.edges.remove(entity);
    }

    fn add_face(&mut self, entity: Entity) {
        self.faces.insert(entity);
    }

    fn remove_face(&mut self, entity: &Entity) {
        self.faces.remove(entity);
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
    // frame entity -> face entity list
    frame_face_indices: HashMap<Entity, Vec<Entity>>,
    // file entity -> file frame data
    file_frame_data: HashMap<Entity, FileFrameData>,
    // frame_entity -> file_entity
    frames: HashMap<Entity, Entity>,
}

impl Default for IconManager {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            faces: HashMap::new(),
            frame_face_indices: HashMap::new(),
            file_frame_data: HashMap::new(),
            frames: HashMap::new(),
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

    pub fn has_frame(&self, frame_entity: &Entity) -> bool {
        self.frames.contains_key(frame_entity)
    }

    pub fn get_frame_at_index(&self, file_entity: &Entity, frame_index: usize) -> Option<Entity> {
        if let Some(file_frame_data) = self.file_frame_data.get(file_entity) {
            file_frame_data.frame_list.get(frame_index).cloned().flatten()
        } else {
            None
        }
    }

    pub fn update_frame_order(
        &mut self,
        file_entity: &Entity,
        frame_entity: &Entity,
        frame_order: u8,
        frame_q: &mut Query<&mut IconFrame>,
    ) {
        info!(
            "icon frame update event (File Entity: {:?}, Frame Entity: {:?}, Order: {:?})",
            file_entity,
            frame_entity,
            frame_order
        );

        let Some(file_frame_data) = self.file_frame_data.get_mut(file_entity) else {
            panic!("file entity not found");
        };
        file_frame_data.update_frame_order(frame_entity, frame_order, frame_q);
    }

    pub fn get_face_index(&self, entity: &Entity) -> Option<usize> {
        if let Some(face_data) = self.faces.get(entity) {
            Some(face_data.face_index)
        } else {
            None
        }
    }

    pub fn on_create_vertex(&mut self, frame_entity: Entity, vertex_entity: Entity) {
        // info!("on_create_vertex: {:?} {:?}", entity, parent_opt);

        info!("inserting icon vert entity: `{:?}`", vertex_entity);

        let Some(file_entity) = self.frames.get(&frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        file_frame_data.add_vertex(frame_entity, vertex_entity);

        self.vertices.insert(vertex_entity, IconVertexData::new(frame_entity));
    }

    pub fn on_create_edge(
        &mut self,
        frame_entity: Entity,
        start_vertex_entity: Entity,
        edge_entity: Entity,
        end_vertex_entity: Entity,
    ) {
        let Some(file_entity) = self.frames.get(&frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        file_frame_data.add_edge(frame_entity, edge_entity);

        self.edges.insert(
            edge_entity,
            IconEdgeData::new(frame_entity, start_vertex_entity, end_vertex_entity),
        );

        for vertex_entity in [start_vertex_entity, end_vertex_entity] {
            let Some(vertex_data) = self.vertices.get_mut(&vertex_entity) else {
                panic!(
                    "on_create_icon_edge: vertex entity `{:?}` not found!",
                    vertex_entity
                );
            };
            vertex_data.add_edge(edge_entity);
        }
    }

    pub fn on_create_face(
        &mut self,
        frame_entity: &Entity,
        old_index_opt: Option<usize>,
        face_entity: Entity,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
    ) {
        let Some(file_entity) = self.frames.get(frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        file_frame_data.add_face(*frame_entity, face_entity);

        // assign index
        let face_index = self.assign_index_to_new_face(frame_entity, old_index_opt, &face_entity);

        self.faces.insert(
            face_entity,
            IconFaceData::new(*frame_entity, face_index, vertex_a, vertex_b, vertex_c),
        );

        // add faces to vertices
        for vertex_entity in [vertex_a, vertex_b, vertex_c] {
            let Some(data) = self.vertices.get_mut(&vertex_entity) else {
                panic!(
                    "on_create_face: vertex entity `{:?}` not found!",
                    vertex_entity
                );
            };
            data.add_face(face_entity);
        }

        // TODO: add face to edges
    }

    pub fn on_create_frame(
        &mut self,
        file_entity: &Entity,
        frame_entity: &Entity,
        frame_index: usize,
        frame_q_opt: Option<&mut Query<&mut IconFrame>>,
    ) {
        if !self.file_frame_data.contains_key(file_entity) {
            self.file_frame_data
                .insert(*file_entity, FileFrameData::new());
        }
        let file_frame_data = self.file_frame_data.get_mut(file_entity).unwrap();
        file_frame_data.add_frame(*frame_entity, frame_index, frame_q_opt);

        self.frames.insert(*frame_entity, *file_entity);
    }

    pub fn on_despawn_frame(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        frame_entity: &Entity,
        frame_q_opt: Option<&mut Query<&mut IconFrame>>,
    ) -> Vec<Entity> {
        let mut despawn_entities = Vec::new();
        let frame_data = self.deregister_frame(frame_entity, frame_q_opt).unwrap();
        for vertex_entity in frame_data.vertices {
            commands
                .entity(vertex_entity)
                .take_authority(server)
                .despawn();
            self.deregister_vertex(&vertex_entity);
            despawn_entities.push(vertex_entity);
        }
        for edge_entity in frame_data.edges {
            commands
                .entity(edge_entity)
                .take_authority(server)
                .despawn();
            self.deregister_edge(&edge_entity);
            despawn_entities.push(edge_entity);
        }
        for face_entity in frame_data.faces {
            commands
                .entity(face_entity)
                .take_authority(server)
                .despawn();
            self.deregister_face(&face_entity);
            despawn_entities.push(face_entity);
        }

        despawn_entities
    }

    pub fn deregister_frame(
        &mut self,
        frame_entity: &Entity,
        frame_q_opt: Option<&mut Query<&mut IconFrame>>,
    ) -> Option<FrameData> {
        let Some(file_entity) = self.frames.remove(frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        let output = file_frame_data.remove_frame(frame_entity, frame_q_opt);
        if file_frame_data.frames.is_empty() {
            self.file_frame_data.remove(&file_entity);
        }

        output
    }

    fn assign_index_to_new_face(
        &mut self,
        frame_entity: &Entity,
        old_index_opt: Option<usize>,
        net_face_entity: &Entity,
    ) -> usize {
        info!(
            "assign_index_to_new_face(entity: `{:?}`, index: {:?})",
            net_face_entity, old_index_opt
        );
        if !self.frame_face_indices.contains_key(frame_entity) {
            self.frame_face_indices.insert(*frame_entity, Vec::new());
        }
        let frame_face_indices = self.frame_face_indices.get_mut(frame_entity).unwrap();

        let new_index = frame_face_indices.len();

        if let Some(old_index) = old_index_opt {
            if new_index != old_index {
                panic!(
                    "something went wrong, got new index `{:?}` but old index was `{:?}`",
                    new_index, old_index
                );
            }
        }

        frame_face_indices.push(*net_face_entity);

        new_index
    }

    pub fn deregister_vertex(&mut self, vertex_entity: &Entity) -> IconVertexData {
        let Some(vert_data) = self.vertices.remove(vertex_entity) else {
            panic!("vertex entity not found");
        };

        let frame_entity = vert_data.frame_entity;
        if let Some(file_entity) = self.frames.get(&frame_entity) {
            if let Some(frame_data) = self.file_frame_data.get_mut(&file_entity) {
                frame_data.remove_vertex(&frame_entity, vertex_entity);
            }
        }

        vert_data
    }

    pub fn deregister_edge(&mut self, edge_entity: &Entity) -> IconEdgeData {
        let Some(edge_data) = self.edges.remove(edge_entity) else {
            panic!("edge entity not found");
        };

        let frame_entity = edge_data.frame_entity;
        if let Some(file_entity) = self.frames.get(&frame_entity) {
            if let Some(frame_data) = self.file_frame_data.get_mut(&file_entity) {
                frame_data.remove_edge(&frame_entity, edge_entity);
            }
        }

        for vertex_entity in [edge_data.vertex_a, edge_data.vertex_b] {
            if let Some(data) = self.vertices.get_mut(&vertex_entity) {
                data.remove_edge(edge_entity);
            }
        }

        edge_data
    }

    pub fn deregister_face(&mut self, face_entity: &Entity) -> Option<IconFaceData> {
        let Some(face_data) = self.faces.remove(face_entity) else {
            return None;
        };

        let frame_entity = face_data.frame_entity;
        if let Some(file_entity) = self.frames.get(&frame_entity) {
            if let Some(frame_data) = self.file_frame_data.get_mut(&file_entity) {
                frame_data.remove_face(&frame_entity, face_entity);
            }
        }

        // remove face from file face list
        let frame_entity = face_data.frame_entity;
        let face_index = face_data.face_index;
        let frame_face_indices = self.frame_face_indices.get_mut(&frame_entity).unwrap();
        frame_face_indices.remove(face_index);
        for i in face_index..frame_face_indices.len() {
            let face_entity = frame_face_indices[i];
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
        let edge_data = self.deregister_edge(edge_entity);

        for vertex_entity in [edge_data.vertex_a, edge_data.vertex_b] {
            if let Some(data) = self.vertices.get_mut(&vertex_entity) {
                info!(
                    "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
                    vertex_entity, edge_entity
                );
                data.remove_edge(edge_entity);
            }
        }

        // TODO: handle any faces that were dependent on this edge?
    }

    pub(crate) fn on_client_despawn_face(&mut self, face_entity: &Entity) {
        let Some(face_data) = self.deregister_face(face_entity) else {
            warn!(
                "face entity `{:?}` not found, perhaps was already despawned?",
                face_entity
            );
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

        // TODO: remove face from edge data
    }
}
