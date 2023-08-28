use std::collections::{HashMap, HashSet};

use bevy_ecs::{entity::Entity, system::Resource};
use bevy_log::info;

use naia_bevy_server::Server;

use vortex_proto::{components::FileTypeValue, resources::FileEntryKey};

use crate::{
    files::ShapeType,
    resources::{project::ProjectKey, GitManager, ShapeManager},
};

pub enum ShapeWaitlistInsert {
    //// shape
    Vertex(Entity),
    //// shape
    VertexRoot(Entity),
    //// parent, edge, child
    Edge(Entity, Entity, Entity),
    //// face
    Face(Entity),
    //// shape, filetype
    FileType(Entity, FileTypeValue),
    //// shape, project key, file key
    OwnedByFile(Entity, ProjectKey, FileEntryKey),
}

enum ShapeData {
    // (ProjectKey, FileKey, Option<Edge, Parent>)
    SkelVertex(ProjectKey, FileEntryKey, Option<(Entity, Entity)>),
    // ProjectKey, FileKey
    SkelEdge(ProjectKey, FileEntryKey),
    // ProjectKey, FileKey
    MeshVertex(ProjectKey, FileEntryKey),
    // (ProjectKey, FileKey, Start, End)
    MeshEdge(ProjectKey, FileEntryKey, Entity, Entity),
    //
    MeshFace(ProjectKey, FileEntryKey),
}

#[derive(Clone)]
pub struct ShapeWaitlistEntry {
    shape: Option<ShapeType>,
    edge_and_parent_opt: Option<Option<(Entity, Entity)>>,
    edge_entities: Option<(Entity, Entity)>,
    file_type: Option<FileTypeValue>,
    owned_by_file: Option<(ProjectKey, FileEntryKey)>,
}

impl ShapeWaitlistEntry {
    fn new() -> Self {
        Self {
            shape: None,
            edge_and_parent_opt: None,
            edge_entities: None,
            file_type: None,
            owned_by_file: None,
        }
    }

    fn is_ready(&self) -> bool {
        match (self.file_type, self.shape) {
            (Some(FileTypeValue::Skel), Some(ShapeType::Vertex)) => {
                return self.owned_by_file.is_some() && self.edge_and_parent_opt.is_some();
            }
            (Some(FileTypeValue::Skel), Some(ShapeType::Edge)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileTypeValue::Mesh), Some(ShapeType::Vertex)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileTypeValue::Mesh), Some(ShapeType::Edge)) => {
                self.owned_by_file.is_some() && self.edge_entities.is_some()
            }
            (Some(FileTypeValue::Mesh), Some(ShapeType::Face)) => {
                return self.owned_by_file.is_some();
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

    fn set_shape_type(&mut self, shape_type: ShapeType) {
        self.shape = Some(shape_type);
    }

    fn set_file_type(&mut self, file_type: FileTypeValue) {
        self.file_type = Some(file_type);
    }

    fn set_owned_by_file(&mut self, project_key: ProjectKey, file_key: FileEntryKey) {
        self.owned_by_file = Some((project_key, file_key));
    }

    fn decompose(self) -> ShapeData {
        let (project_key, file_key) = self.owned_by_file.unwrap();

        match (self.file_type, self.shape) {
            (Some(FileTypeValue::Skel), Some(ShapeType::Vertex)) => {
                return ShapeData::SkelVertex(
                    project_key,
                    file_key,
                    self.edge_and_parent_opt.unwrap(),
                );
            }
            (Some(FileTypeValue::Skel), Some(ShapeType::Edge)) => {
                return ShapeData::SkelEdge(project_key, file_key);
            }
            (Some(FileTypeValue::Mesh), Some(ShapeType::Vertex)) => {
                return ShapeData::MeshVertex(project_key, file_key);
            }
            (Some(FileTypeValue::Mesh), Some(ShapeType::Edge)) => {
                let (start, end) = self.edge_entities.unwrap();
                return ShapeData::MeshEdge(project_key, file_key, start, end);
            }
            (Some(FileTypeValue::Mesh), Some(ShapeType::Face)) => {
                return ShapeData::MeshFace(project_key, file_key);
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
    // waiting entity -> (entity dependencies, entry)
    dependent_map: HashMap<Entity, (HashSet<Entity>, ShapeWaitlistEntry)>,
    // entity dependency -> entities waiting on it
    dependency_map: HashMap<Entity, HashSet<Entity>>,
}

impl Default for ShapeWaitlist {
    fn default() -> Self {
        Self {
            incomplete_entries: HashMap::new(),
            dependent_map: HashMap::new(),
            dependency_map: HashMap::new(),
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
            ShapeWaitlistInsert::Edge(parent_entity, edge_entity, vertex_entity) => {
                {
                    if !self.contains_key(&vertex_entity) {
                        self.insert_incomplete(vertex_entity, ShapeWaitlistEntry::new());
                    }
                    let vertex_entry = self.get_mut(&vertex_entity).unwrap();
                    info!(
                        "Setting parent of {:?} to {:?}",
                        vertex_entity, parent_entity
                    );
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

                info!(
                    "Entities to check for readiness... `{:?}`",
                    possibly_ready_entities
                );
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
            ShapeWaitlistInsert::Face(face_entity) => {
                if !self.contains_key(&face_entity) {
                    self.insert_incomplete(face_entity, ShapeWaitlistEntry::new());
                }
                let mut entry = self.get_mut(&face_entity).unwrap();
                entry.set_shape_type(ShapeType::Face);
                entry.set_file_type(FileTypeValue::Mesh);
                possibly_ready_entities.push(face_entity);
            }
        }

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
                    (FileTypeValue::Skel, ShapeType::Vertex) => {
                        if entry.has_edge_and_parent() {
                            let (_, parent_entity) = entry.get_edge_and_parent().unwrap();
                            if !shape_manager.has_vertex(&parent_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "vert entity {:?} requires parent {:?}. putting in parent waitlist",
                                    entity,
                                    parent_entity
                                );
                                self.insert_waiting_dependency(parent_entity, entity, entry);
                                continue;
                            }
                        }
                    }
                    (FileTypeValue::Skel, ShapeType::Edge) => {
                        info!("`{:?}` Skel Edge complete!", entity);
                        return;
                    }
                    (FileTypeValue::Mesh, ShapeType::Vertex) => {
                        info!("`{:?}` Mesh Vertex complete!", entity);
                    }
                    (FileTypeValue::Mesh, ShapeType::Edge) => {
                        let entities = entry.edge_entities.unwrap();
                        let mut has_all_entities = true;
                        for vertex_entity in [&entities.0, &entities.1] {
                            if !shape_manager.has_vertex(vertex_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "edge entity {:?} requires parent {:?}. putting in parent waitlist",
                                    entity, vertex_entity
                                );
                                self.insert_waiting_dependency(*vertex_entity, entity, entry.clone());
                                has_all_entities = false;
                            }
                        }
                        if !has_all_entities {
                            continue;
                        }
                    }
                    (FileTypeValue::Mesh, ShapeType::Face) => {
                        todo!();
                    }
                    (FileTypeValue::Skel, ShapeType::Face) => {
                        panic!("not possible");
                    }
                }

                info!("processing shape {:?}", entity);
                self.process_complete(server, git_manager, shape_manager, entity, entry);
            } else {
                info!("entity `{:?}` is not ready yet...", possibly_ready_entity);
            }
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

        match data {
            ShapeData::SkelVertex(project_key, file_key, edge_and_parent_opt) => {
                git_manager.on_client_insert_content_entity(
                    server,
                    &project_key,
                    &file_key,
                    &entity,
                    ShapeType::Vertex,
                );
                shape_manager.on_create_skel_vertex(entity, edge_and_parent_opt);
            }
            ShapeData::SkelEdge(project_key, file_key) => {
                git_manager.on_client_insert_content_entity(
                    server,
                    &project_key,
                    &file_key,
                    &entity,
                    ShapeType::Edge,
                );
            }
            ShapeData::MeshVertex(project_key, file_key) => {
                git_manager.on_client_insert_content_entity(
                    server,
                    &project_key,
                    &file_key,
                    &entity,
                    ShapeType::Vertex,
                );
                shape_manager.on_create_mesh_vertex(entity);
            }
            ShapeData::MeshEdge(project_key, file_key, start, end) => {
                git_manager.on_client_insert_content_entity(
                    server,
                    &project_key,
                    &file_key,
                    &entity,
                    ShapeType::Edge,
                );
                shape_manager.on_create_mesh_edge(start, entity, end);
            }
            ShapeData::MeshFace(project_key, file_key) => {
                todo!()
            }
        }

        // if the waitlist has any children entities of this one, process them
        info!(
            "processing complete shape {:?}. checking for children",
            entity
        );
        if let Some(child_entries) = self.on_vertex_complete(entity) {
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

    fn insert_waiting_dependency(
        &mut self,
        dependency_entity: Entity,
        dependent_entity: Entity,
        dependent_entry: ShapeWaitlistEntry,
    ) {
        if !self.dependency_map.contains_key(&dependency_entity) {
            self.dependency_map
                .insert(dependency_entity, HashSet::new());
        }
        let dependents = self.dependency_map.get_mut(&dependency_entity).unwrap();
        dependents.insert(dependent_entity);

        if !self.dependent_map.contains_key(&dependent_entity) {
            self.dependent_map
                .insert(dependent_entity, (HashSet::new(), dependent_entry));
        }
        let (dependencies, _) = self.dependent_map.get_mut(&dependent_entity).unwrap();
        dependencies.insert(dependency_entity);
    }

    fn on_vertex_complete(&mut self, entity: Entity) -> Option<Vec<(Entity, ShapeWaitlistEntry)>> {
        if let Some(dependents) = self.dependency_map.remove(&entity) {
            let mut result = Vec::new();
            for dependent in dependents {
                let (dependencies, _) = self.dependent_map.get_mut(&dependent).unwrap();
                dependencies.remove(&entity);
                if dependencies.is_empty() {
                    let (_, entry) = self.dependent_map.remove(&dependent).unwrap();
                    result.push((dependent, entry));
                }
            }
            Some(result)
        } else {
            None
        }
    }
}
