use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};
use bevy_log::info;

use naia_bevy_server::Server;

use vortex_proto::{
    components::FileExtension,
    resources::{DependencyMap, FileKey},
};

use crate::{
    files::ShapeType,
    resources::{project::ProjectKey, ContentEntityData, GitManager, ShapeManager},
};

pub enum ShapeWaitlistInsert {
    //// shape
    Vertex(Entity),
    //// shape
    VertexRoot(Entity),
    //// parent, edge, child
    Edge(Entity, Entity, Entity),
    //// (face_entity, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c)
    Face(Entity, Entity, Entity, Entity),
    //// shape, filetype
    FileType(Entity, FileExtension),
    //// shape, project key, file key
    OwnedByFile(Entity, ProjectKey, FileKey),
}

enum ShapeData {
    // (ProjectKey, FileKey, Option<Edge, Parent>)
    SkelVertex(ProjectKey, FileKey, Option<(Entity, Entity)>),
    // ProjectKey, FileKey
    SkelEdge(ProjectKey, FileKey),
    // ProjectKey, FileKey
    MeshVertex(ProjectKey, FileKey),
    // (ProjectKey, FileKey, Start, End)
    MeshEdge(ProjectKey, FileKey, Entity, Entity),
    // (ProjectKey, FileKey, VertexA, VertexB, VertexC, EdgeA, EdgeB, EdgeC)
    MeshFace(ProjectKey, FileKey, Entity, Entity, Entity),
}

#[derive(Clone)]
pub struct ShapeWaitlistEntry {
    shape: Option<ShapeType>,
    edge_and_parent_opt: Option<Option<(Entity, Entity)>>,
    edge_entities: Option<(Entity, Entity)>,
    face_entities: Option<(Entity, Entity, Entity)>,
    file_type: Option<FileExtension>,
    owned_by_file: Option<(ProjectKey, FileKey)>,
}

impl ShapeWaitlistEntry {
    fn new() -> Self {
        Self {
            shape: None,
            edge_and_parent_opt: None,
            edge_entities: None,
            face_entities: None,
            file_type: None,
            owned_by_file: None,
        }
    }

    fn is_ready(&self) -> bool {
        match (self.file_type, self.shape) {
            (Some(FileExtension::Skel), Some(ShapeType::Vertex)) => {
                return self.owned_by_file.is_some() && self.edge_and_parent_opt.is_some();
            }
            (Some(FileExtension::Skel), Some(ShapeType::Edge)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileExtension::Mesh), Some(ShapeType::Vertex)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileExtension::Mesh), Some(ShapeType::Edge)) => {
                return self.owned_by_file.is_some() && self.edge_entities.is_some();
            }
            (Some(FileExtension::Mesh), Some(ShapeType::Face)) => {
                return self.owned_by_file.is_some() && self.face_entities.is_some();
            }
            _ => {
                return false;
            }
        }
    }

    fn set_edge_and_parent(&mut self, parent: Option<(Entity, Entity)>) {
        self.edge_and_parent_opt = Some(parent);
    }

    fn get_edge_and_parent(&self) -> Option<(Entity, Entity)> {
        self.edge_and_parent_opt.unwrap()
    }

    fn has_edge_and_parent(&self) -> bool {
        if let Some(parent_opt) = &self.edge_and_parent_opt {
            return parent_opt.is_some();
        }
        return false;
    }

    fn set_edge_entities(&mut self, start: Entity, end: Entity) {
        self.edge_entities = Some((start, end));
    }

    fn set_face_entities(&mut self, vertex_a: Entity, vertex_b: Entity, vertex_c: Entity) {
        self.face_entities = Some((vertex_a, vertex_b, vertex_c));
    }

    fn set_shape_type(&mut self, shape_type: ShapeType) {
        self.shape = Some(shape_type);
    }

    fn set_file_type(&mut self, file_type: FileExtension) {
        self.file_type = Some(file_type);
    }

    fn set_owned_by_file(&mut self, project_key: ProjectKey, file_key: FileKey) {
        self.owned_by_file = Some((project_key, file_key));
    }

    fn decompose(self) -> ShapeData {
        let (project_key, file_key) = self.owned_by_file.unwrap();

        match (self.file_type, self.shape) {
            (Some(FileExtension::Skel), Some(ShapeType::Vertex)) => {
                return ShapeData::SkelVertex(
                    project_key,
                    file_key,
                    self.edge_and_parent_opt.unwrap(),
                );
            }
            (Some(FileExtension::Skel), Some(ShapeType::Edge)) => {
                return ShapeData::SkelEdge(project_key, file_key);
            }
            (Some(FileExtension::Mesh), Some(ShapeType::Vertex)) => {
                return ShapeData::MeshVertex(project_key, file_key);
            }
            (Some(FileExtension::Mesh), Some(ShapeType::Edge)) => {
                let (start, end) = self.edge_entities.unwrap();
                return ShapeData::MeshEdge(project_key, file_key, start, end);
            }
            (Some(FileExtension::Mesh), Some(ShapeType::Face)) => {
                let (vertex_a, vertex_b, vertex_c) = self.face_entities.unwrap();
                return ShapeData::MeshFace(project_key, file_key, vertex_a, vertex_b, vertex_c);
            }
            _ => {
                panic!("shouldn't be able to happen!");
            }
        }
    }
}

#[derive(Resource)]
pub struct ShapeWaitlist {
    // incomplete entity -> entry
    incomplete_entries: HashMap<Entity, ShapeWaitlistEntry>,
    dependency_map: DependencyMap<Entity, ShapeWaitlistEntry>,
}

impl Default for ShapeWaitlist {
    fn default() -> Self {
        Self {
            incomplete_entries: HashMap::new(),
            dependency_map: DependencyMap::new(),
        }
    }
}

impl ShapeWaitlist {
    pub fn process_insert(
        &mut self,
        server: &mut Server,
        git_manager: &mut GitManager,
        shape_manager: &mut ShapeManager,
        insert: ShapeWaitlistInsert,
    ) {
        let mut possibly_ready_entities = Vec::new();

        match insert {
            ShapeWaitlistInsert::Vertex(vertex_entity) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, ShapeWaitlistEntry::new());
                }
                self.get_mut(&vertex_entity)
                    .unwrap()
                    .set_shape_type(ShapeType::Vertex);
            }
            ShapeWaitlistInsert::VertexRoot(vertex_entity) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, ShapeWaitlistEntry::new());
                }
                let entry = self.get_mut(&vertex_entity).unwrap();
                entry.set_edge_and_parent(None);
                entry.set_shape_type(ShapeType::Vertex);
                possibly_ready_entities.push(vertex_entity);
            }
            ShapeWaitlistInsert::Edge(parent_entity, edge_entity, vertex_entity) => {
                {
                    if !self.contains_key(&vertex_entity) {
                        self.insert_incomplete(vertex_entity, ShapeWaitlistEntry::new());
                    }
                    let vertex_entry = self.get_mut(&vertex_entity).unwrap();
                    // info!(
                    //     "Setting parent of {:?} to {:?}",
                    //     vertex_entity, parent_entity
                    // );
                    vertex_entry.set_edge_and_parent(Some((edge_entity, parent_entity)));
                    vertex_entry.set_shape_type(ShapeType::Vertex);
                    possibly_ready_entities.push(vertex_entity);
                }

                {
                    if !self.contains_key(&edge_entity) {
                        self.insert_incomplete(edge_entity, ShapeWaitlistEntry::new());
                    }
                    let edge_entry = self.get_mut(&edge_entity).unwrap();
                    edge_entry.set_shape_type(ShapeType::Edge);
                    edge_entry.set_edge_entities(parent_entity, vertex_entity);
                    possibly_ready_entities.push(edge_entity);
                }

                // info!(
                //     "Entities to check for readiness... `{:?}`",
                //     possibly_ready_entities
                // );
            }
            ShapeWaitlistInsert::Face(face_entity, vertex_a, vertex_b, vertex_c) => {
                if !self.contains_key(&face_entity) {
                    self.insert_incomplete(face_entity, ShapeWaitlistEntry::new());
                }
                let entry = self.get_mut(&face_entity).unwrap();
                entry.set_shape_type(ShapeType::Face);
                entry.set_file_type(FileExtension::Mesh);
                entry.set_face_entities(vertex_a, vertex_b, vertex_c);
                possibly_ready_entities.push(face_entity);
            }
            ShapeWaitlistInsert::FileType(shape_entity, file_type) => {
                if !self.contains_key(&shape_entity) {
                    self.insert_incomplete(shape_entity, ShapeWaitlistEntry::new());
                }
                self.get_mut(&shape_entity)
                    .unwrap()
                    .set_file_type(file_type);
                possibly_ready_entities.push(shape_entity);
            }
            ShapeWaitlistInsert::OwnedByFile(shape_entity, project_key, file_key) => {
                if !self.contains_key(&shape_entity) {
                    self.insert_incomplete(shape_entity, ShapeWaitlistEntry::new());
                }
                self.get_mut(&shape_entity)
                    .unwrap()
                    .set_owned_by_file(project_key, file_key);
                possibly_ready_entities.push(shape_entity);
            }
        }

        let mut entities_to_process = Vec::new();
        for possibly_ready_entity in possibly_ready_entities {
            if self
                .incomplete_entries
                .get(&possibly_ready_entity)
                .unwrap()
                .is_ready()
            {
                let entity = possibly_ready_entity;
                info!("entity `{:?}` is ready!", entity);
                let entry = self.remove(&entity).unwrap();

                match (entry.file_type.unwrap(), entry.shape.unwrap()) {
                    (FileExtension::Skel, ShapeType::Vertex) => {
                        if entry.has_edge_and_parent() {
                            let (_, parent_entity) = entry.get_edge_and_parent().unwrap();
                            if !shape_manager.has_vertex(&parent_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "vert entity {:?} requires parent {:?}. putting in parent waitlist",
                                    entity,
                                    parent_entity
                                );
                                self.dependency_map.insert_waiting_dependencies(
                                    vec![parent_entity],
                                    entity,
                                    entry,
                                );
                                continue;
                            }
                        }
                    }
                    (FileExtension::Skel, ShapeType::Edge) => {
                        info!("`{:?}` Skel Edge complete!", entity);
                    }
                    (FileExtension::Mesh, ShapeType::Vertex) => {
                        info!("`{:?}` Mesh Vertex complete!", entity);
                    }
                    (FileExtension::Mesh, ShapeType::Edge) => {
                        let edge_entities = entry.edge_entities.unwrap();
                        let mut dependencies = Vec::new();

                        for vertex_entity in [&edge_entities.0, &edge_entities.1] {
                            if !shape_manager.has_vertex(vertex_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "edge entity {:?} requires parent {:?}. putting in parent waitlist",
                                    entity, vertex_entity
                                );
                                dependencies.push(*vertex_entity);
                            }
                        }

                        if !dependencies.is_empty() {
                            self.dependency_map.insert_waiting_dependencies(
                                dependencies,
                                entity,
                                entry,
                            );
                            continue;
                        }
                    }
                    (FileExtension::Mesh, ShapeType::Face) => {
                        info!("`{:?}` Mesh Face complete!", entity);
                    }
                    (FileExtension::Skel, ShapeType::Face) => {
                        panic!("not possible");
                    }
                    (_, _) => {
                        panic!("not possible");
                    }
                }

                info!(
                    "processing shape type: `{:?}`, entity: `{:?}`",
                    entry.shape.unwrap(),
                    entity
                );
                entities_to_process.push((entity, entry));
            } else {
                info!("entity `{:?}` is not ready yet...", possibly_ready_entity);
            }
        }

        for (entity, entry) in entities_to_process {
            self.process_complete(server, git_manager, shape_manager, entity, entry);
        }
    }

    fn process_complete(
        &mut self,
        server: &mut Server,
        git_manager: &mut GitManager,
        shape_manager: &mut ShapeManager,
        entity: Entity,
        entry: ShapeWaitlistEntry,
    ) {
        // info!("processing complete vertex {:?}", entity);

        let data = entry.decompose();

        let (project_key, file_key, shape_type) = match data {
            ShapeData::SkelVertex(project_key, file_key, edge_and_parent_opt) => {
                shape_manager.on_create_skel_vertex(entity, edge_and_parent_opt);
                (project_key, file_key, ShapeType::Vertex)
            }
            ShapeData::SkelEdge(project_key, file_key) => (project_key, file_key, ShapeType::Edge),
            ShapeData::MeshVertex(project_key, file_key) => {
                shape_manager.on_create_mesh_vertex(entity);
                (project_key, file_key, ShapeType::Vertex)
            }
            ShapeData::MeshEdge(project_key, file_key, start, end) => {
                shape_manager.on_create_mesh_edge(start, entity, end);
                (project_key, file_key, ShapeType::Vertex)
            }
            ShapeData::MeshFace(project_key, file_key, vertex_a, vertex_b, vertex_c) => {
                let file_entity = git_manager.file_entity(&project_key, &file_key).unwrap();
                shape_manager.on_create_face(
                    &file_entity,
                    None,
                    entity,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                );
                (project_key, file_key, ShapeType::Face)
            }
        };

        let content_entity_data = ContentEntityData::new_shape(shape_type);
        git_manager.on_insert_content_entity(
            server,
            &project_key,
            &file_key,
            &entity,
            &content_entity_data,
        );
        git_manager.queue_client_modify_file(&entity);

        // if the waitlist has any children entities of this one, process them
        info!(
            "processing complete shape {:?}. checking for children",
            entity
        );
        if let Some(child_entries) = self.dependency_map.on_dependency_complete(entity) {
            for (child_entity, child_entry) in child_entries {
                info!(
                    "entity {:?} was waiting on parent {:?}. processing!",
                    child_entity, entity
                );
                self.process_complete(
                    server,
                    git_manager,
                    shape_manager,
                    child_entity,
                    child_entry,
                );
            }
        }
    }

    fn contains_key(&self, entity: &Entity) -> bool {
        self.incomplete_entries.contains_key(entity)
    }

    fn insert_incomplete(&mut self, entity: Entity, entry: ShapeWaitlistEntry) {
        self.incomplete_entries.insert(entity, entry);
    }

    fn get_mut(&mut self, entity: &Entity) -> Option<&mut ShapeWaitlistEntry> {
        self.incomplete_entries.get_mut(entity)
    }

    fn remove(&mut self, entity: &Entity) -> Option<ShapeWaitlistEntry> {
        self.incomplete_entries.remove(entity)
    }
}
