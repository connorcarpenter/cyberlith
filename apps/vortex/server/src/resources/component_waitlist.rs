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
    resources::{project::ProjectKey, ContentEntityData, GitManager, IconManager, ShapeManager},
};

#[derive(Clone)]
pub enum ComponentWaitlistInsert {
    //// filetype
    FileType(FileExtension),
    //// project key, file key
    OwnedByFile(ProjectKey, FileKey),
    //// shape
    Vertex,
    //// shape
    VertexRoot,
    //// parent, child
    Edge(Entity, Entity),
    //// (option<FrameEntity>, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c)
    Face(
        Option<Entity>,
        Entity,
        Entity,
        Entity,
        Entity,
        Entity,
        Entity,
    ),
    ////
    NetTransform,
    ////
    SkinOrSceneEntity,
    ////
    ShapeName,
}

#[derive(Clone, Copy, Debug)]
enum ComponentType {
    Vertex,
    Edge,
    Face,
    NetTransform,
}

enum ComponentData {
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
    // ProjectKey, FileKey
    IconVertex(ProjectKey, FileKey),
    // (ProjectKey, FileKey, Start, End)
    IconEdge(ProjectKey, FileKey, Entity, Entity),
    // (ProjectKey, FileKey, FrameEntity, VertexA, VertexB, VertexC, EdgeA, EdgeB, EdgeC)
    IconFace(ProjectKey, FileKey, Entity, Entity, Entity, Entity),
    //
    ModelTransform(ProjectKey, FileKey),
    //
    SceneTransform(ProjectKey, FileKey),
}

#[derive(Clone)]
pub struct ComponentWaitlistEntry {
    component_type: Option<ComponentType>,
    file_type: Option<FileExtension>,
    owned_by_file: Option<(ProjectKey, FileKey)>,

    edge_and_parent_opt: Option<Option<(Entity, Entity)>>,
    edge_entities: Option<(Entity, Entity)>,
    // Option<(Option<frame entity>, vertex a entity, vertex b entity, vertex c entity, edge a entity, edge b entity, edge c entity)
    face_entities: Option<(
        Option<Entity>,
        Entity,
        Entity,
        Entity,
        Entity,
        Entity,
        Entity,
    )>,

    skin_or_scene_entity: bool,
    shape_name: bool,
}

impl ComponentWaitlistEntry {
    fn new() -> Self {
        Self {
            component_type: None,
            file_type: None,
            owned_by_file: None,

            edge_and_parent_opt: None,
            edge_entities: None,
            face_entities: None,

            skin_or_scene_entity: false,
            shape_name: false,
        }
    }

    fn is_ready(&self) -> bool {
        match (self.file_type, self.component_type) {
            (Some(FileExtension::Skel), Some(ComponentType::Vertex)) => {
                return self.owned_by_file.is_some() && self.edge_and_parent_opt.is_some();
            }
            (Some(FileExtension::Skel), Some(ComponentType::Edge)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Vertex)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Edge)) => {
                return self.owned_by_file.is_some() && self.edge_entities.is_some();
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Face)) => {
                return self.owned_by_file.is_some() && self.face_entities.is_some();
            }
            (Some(FileExtension::Model), Some(ComponentType::NetTransform)) => {
                return self.owned_by_file.is_some() && self.skin_or_scene_entity && self.shape_name
            }
            (Some(FileExtension::Scene), Some(ComponentType::NetTransform)) => {
                return self.owned_by_file.is_some() && self.skin_or_scene_entity
            }
            (Some(FileExtension::Icon), Some(ComponentType::Vertex)) => {
                return self.owned_by_file.is_some();
            }
            (Some(FileExtension::Icon), Some(ComponentType::Edge)) => {
                return self.owned_by_file.is_some() && self.edge_entities.is_some();
            }
            (Some(FileExtension::Icon), Some(ComponentType::Face)) => {
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

    fn set_face_entities(
        &mut self,
        frame_entity: Option<Entity>,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
        edge_a: Entity,
        edge_b: Entity,
        edge_c: Entity,
    ) {
        self.face_entities = Some((
            frame_entity,
            vertex_a,
            vertex_b,
            vertex_c,
            edge_a,
            edge_b,
            edge_c,
        ));
    }

    fn set_component_type(&mut self, component_type: ComponentType) {
        self.component_type = Some(component_type);
    }

    fn set_file_type(&mut self, file_type: FileExtension) {
        self.file_type = Some(file_type);
    }

    fn set_owned_by_file(&mut self, project_key: ProjectKey, file_key: FileKey) {
        self.owned_by_file = Some((project_key, file_key));
    }

    fn set_transform(&mut self) {
        self.component_type = Some(ComponentType::NetTransform);
    }

    fn set_skin_or_scene_entity(&mut self) {
        self.skin_or_scene_entity = true;
    }

    fn set_shape_name(&mut self) {
        self.shape_name = true;
    }

    fn decompose(self) -> ComponentData {
        let (project_key, file_key) = self.owned_by_file.unwrap();

        match (self.file_type, self.component_type) {
            (Some(FileExtension::Skel), Some(ComponentType::Vertex)) => {
                return ComponentData::SkelVertex(
                    project_key,
                    file_key,
                    self.edge_and_parent_opt.unwrap(),
                );
            }
            (Some(FileExtension::Skel), Some(ComponentType::Edge)) => {
                return ComponentData::SkelEdge(project_key, file_key);
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Vertex)) => {
                return ComponentData::MeshVertex(project_key, file_key);
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Edge)) => {
                let (start, end) = self.edge_entities.unwrap();
                return ComponentData::MeshEdge(project_key, file_key, start, end);
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Face)) => {
                let (None, vertex_a, vertex_b, vertex_c, _, _, _) = self.face_entities.unwrap() else {
                    panic!("invalid");
                };
                return ComponentData::MeshFace(
                    project_key,
                    file_key,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                );
            }
            (Some(FileExtension::Model), Some(ComponentType::NetTransform)) => {
                return ComponentData::ModelTransform(project_key, file_key);
            }
            (Some(FileExtension::Scene), Some(ComponentType::NetTransform)) => {
                return ComponentData::SceneTransform(project_key, file_key);
            }
            (Some(FileExtension::Icon), Some(ComponentType::Vertex)) => {
                return ComponentData::IconVertex(project_key, file_key);
            }
            (Some(FileExtension::Icon), Some(ComponentType::Edge)) => {
                let (start, end) = self.edge_entities.unwrap();
                return ComponentData::IconEdge(project_key, file_key, start, end);
            }
            (Some(FileExtension::Icon), Some(ComponentType::Face)) => {
                let (Some(frame_entity), vertex_a, vertex_b, vertex_c, _, _, _) = self.face_entities.unwrap() else {
                    panic!("invalid");
                };
                return ComponentData::IconFace(
                    project_key,
                    file_key,
                    frame_entity,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                );
            }
            _ => {
                panic!("shouldn't be able to happen!");
            }
        }
    }
}

#[derive(Resource)]
pub struct ComponentWaitlist {
    // incomplete entity -> entry
    incomplete_entries: HashMap<Entity, ComponentWaitlistEntry>,
    dependency_map: DependencyMap<Entity, ComponentWaitlistEntry>,
}

impl Default for ComponentWaitlist {
    fn default() -> Self {
        Self {
            incomplete_entries: HashMap::new(),
            dependency_map: DependencyMap::new(),
        }
    }
}

impl ComponentWaitlist {
    pub fn process_inserts(
        &mut self,
        server: &mut Server,
        git_manager: &mut GitManager,
        shape_manager_opt: &mut Option<&mut ShapeManager>,
        icon_manager_opt: &mut Option<&mut IconManager>,
        entity: &Entity,
        inserts: &[ComponentWaitlistInsert],
    ) {
        for insert in inserts {
            self.process_insert(
                server,
                git_manager,
                shape_manager_opt,
                icon_manager_opt,
                entity,
                insert.clone(),
            );
        }
    }

    pub fn process_insert(
        &mut self,
        server: &mut Server,
        git_manager: &mut GitManager,
        shape_manager_opt: &mut Option<&mut ShapeManager>,
        icon_manager_opt: &mut Option<&mut IconManager>,
        entity: &Entity,
        insert: ComponentWaitlistInsert,
    ) {
        let mut possibly_ready_entities = Vec::new();

        possibly_ready_entities.push(*entity);

        match insert {
            ComponentWaitlistInsert::Vertex => {
                if !self.contains_key(entity) {
                    self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
                }
                self.get_mut(entity)
                    .unwrap()
                    .set_component_type(ComponentType::Vertex);
            }
            ComponentWaitlistInsert::VertexRoot => {
                if !self.contains_key(entity) {
                    self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
                }
                let entry = self.get_mut(entity).unwrap();
                entry.set_edge_and_parent(None);
                entry.set_component_type(ComponentType::Vertex);
            }
            ComponentWaitlistInsert::Edge(parent_entity, vertex_entity) => {
                {
                    if !self.contains_key(&vertex_entity) {
                        self.insert_incomplete(vertex_entity, ComponentWaitlistEntry::new());
                    }
                    let vertex_entry = self.get_mut(&vertex_entity).unwrap();
                    // info!(
                    //     "Setting parent of {:?} to {:?}",
                    //     vertex_entity, parent_entity
                    // );
                    vertex_entry.set_edge_and_parent(Some((*entity, parent_entity)));
                    vertex_entry.set_component_type(ComponentType::Vertex);
                    possibly_ready_entities.push(vertex_entity);
                }

                {
                    if !self.contains_key(entity) {
                        self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
                    }
                    let edge_entry = self.get_mut(entity).unwrap();
                    edge_entry.set_component_type(ComponentType::Edge);
                    edge_entry.set_edge_entities(parent_entity, vertex_entity);
                }

                // info!(
                //     "Entities to check for readiness... `{:?}`",
                //     possibly_ready_entities
                // );
            }
            ComponentWaitlistInsert::Face(
                frame_entity_opt,
                vertex_a,
                vertex_b,
                vertex_c,
                edge_a,
                edge_b,
                edge_c,
            ) => {
                if !self.contains_key(entity) {
                    self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
                }
                let entry = self.get_mut(entity).unwrap();
                entry.set_component_type(ComponentType::Face);
                entry.set_face_entities(
                    frame_entity_opt,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                    edge_a,
                    edge_b,
                    edge_c,
                );
            }
            ComponentWaitlistInsert::FileType(file_type) => {
                if !self.contains_key(entity) {
                    self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
                }
                self.get_mut(entity).unwrap().set_file_type(file_type);
            }
            ComponentWaitlistInsert::OwnedByFile(project_key, file_key) => {
                if !self.contains_key(entity) {
                    self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
                }
                self.get_mut(entity)
                    .unwrap()
                    .set_owned_by_file(project_key, file_key);
            }
            ComponentWaitlistInsert::SkinOrSceneEntity => {
                self.get_mut(entity).unwrap().set_skin_or_scene_entity();
            }
            ComponentWaitlistInsert::ShapeName => {
                self.get_mut(entity).unwrap().set_shape_name();
            }
            ComponentWaitlistInsert::NetTransform => {
                self.get_mut(entity).unwrap().set_transform();
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

                match (entry.file_type.unwrap(), entry.component_type.unwrap()) {
                    (FileExtension::Skel, ComponentType::Vertex) => {
                        if entry.has_edge_and_parent() {
                            let Some(shape_manager) = shape_manager_opt else {
                                panic!("shape manager not available");
                            };

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
                    (FileExtension::Skel, ComponentType::Edge) => {
                        info!("`{:?}` Skel Edge complete!", entity);
                    }
                    (FileExtension::Mesh, ComponentType::Vertex) => {
                        info!("`{:?}` Mesh Vertex complete!", entity);
                    }
                    (FileExtension::Mesh, ComponentType::Edge) => {
                        let edge_entities = entry.edge_entities.unwrap();
                        let mut dependencies = Vec::new();
                        let Some(shape_manager) = shape_manager_opt else {
                            panic!("shape manager not available");
                        };

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
                    (FileExtension::Mesh, ComponentType::Face) => {
                        let face_entities = entry.face_entities.unwrap();
                        let mut dependencies = Vec::new();
                        let Some(shape_manager) = shape_manager_opt else {
                            panic!("shape manager not available");
                        };

                        for vertex_entity in [&face_entities.1, &face_entities.2, &face_entities.3]
                        {
                            if !shape_manager.has_vertex(vertex_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "face entity {:?} requires parent vertex {:?}. putting in parent waitlist",
                                    entity, vertex_entity
                                );
                                dependencies.push(*vertex_entity);
                            }
                        }

                        for edge_entity in [&face_entities.4, &face_entities.5, &face_entities.6] {
                            if !shape_manager.has_edge(edge_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "face entity {:?} requires parent edge {:?}. putting in parent waitlist",
                                    entity, edge_entity
                                );
                                dependencies.push(*edge_entity);
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
                    (FileExtension::Icon, ComponentType::Vertex) => {
                        info!("`{:?}` Icon Vertex complete!", entity);
                    }
                    (FileExtension::Icon, ComponentType::Edge) => {
                        let edge_entities = entry.edge_entities.unwrap();
                        let mut dependencies = Vec::new();
                        let Some(icon_manager) = icon_manager_opt else {
                            panic!("icon manager not available");
                        };

                        for vertex_entity in [&edge_entities.0, &edge_entities.1] {
                            if !icon_manager.has_vertex(vertex_entity) {
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
                    (FileExtension::Icon, ComponentType::Face) => {
                        let face_entities = entry.face_entities.unwrap();
                        let mut dependencies = Vec::new();
                        let Some(icon_manager) = icon_manager_opt else {
                            panic!("icon manager not available");
                        };

                        for vertex_entity in [&face_entities.1, &face_entities.2, &face_entities.3]
                        {
                            if !icon_manager.has_vertex(vertex_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "face entity {:?} requires parent vertex {:?}. putting in parent waitlist",
                                    entity, vertex_entity
                                );
                                dependencies.push(*vertex_entity);
                            }
                        }

                        for edge_entity in [&face_entities.4, &face_entities.5, &face_entities.6] {
                            if !icon_manager.has_edge(edge_entity) {
                                // need to put in parent waitlist
                                info!(
                                    "face entity {:?} requires parent edge {:?}. putting in parent waitlist",
                                    entity, edge_entity
                                );
                                dependencies.push(*edge_entity);
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
                    (FileExtension::Skel, ComponentType::Face) => {
                        panic!("not possible");
                    }
                    (FileExtension::Model, ComponentType::NetTransform) => {
                        info!("`{:?}` Model Transform complete!", entity);
                    }
                    (FileExtension::Scene, ComponentType::NetTransform) => {
                        info!("`{:?}` Scene Transform complete!", entity);
                    }
                    (_, _) => {
                        panic!("not possible");
                    }
                }

                info!(
                    "processing shape type: `{:?}`, entity: `{:?}`",
                    entry.component_type.unwrap(),
                    entity
                );
                entities_to_process.push((entity, entry));
            } else {
                info!("entity `{:?}` is not ready yet...", possibly_ready_entity);
            }
        }

        for (entity, entry) in entities_to_process {
            self.process_complete(
                server,
                git_manager,
                shape_manager_opt,
                icon_manager_opt,
                entity,
                entry,
            );
        }
    }

    fn process_complete(
        &mut self,
        server: &mut Server,
        git_manager: &mut GitManager,
        shape_manager_opt: &mut Option<&mut ShapeManager>,
        icon_manager_opt: &mut Option<&mut IconManager>,
        entity: Entity,
        entry: ComponentWaitlistEntry,
    ) {
        // info!("processing complete vertex {:?}", entity);

        let data = entry.decompose();

        let (project_key, file_key, content_entity_data) = match data {
            ComponentData::SkelVertex(project_key, file_key, edge_and_parent_opt) => {
                let Some(shape_manager) = shape_manager_opt else {
                    panic!("shape manager not available");
                };
                shape_manager.on_create_skel_vertex(entity, edge_and_parent_opt);
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_shape(ShapeType::Vertex),
                )
            }
            ComponentData::SkelEdge(project_key, file_key) => (
                project_key,
                file_key,
                ContentEntityData::new_shape(ShapeType::Edge),
            ),
            ComponentData::MeshVertex(project_key, file_key) => {
                let Some(shape_manager) = shape_manager_opt else {
                    panic!("shape manager not available");
                };
                shape_manager.on_create_mesh_vertex(entity);
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_shape(ShapeType::Vertex),
                )
            }
            ComponentData::MeshEdge(project_key, file_key, start, end) => {
                let Some(shape_manager) = shape_manager_opt else {
                    panic!("shape manager not available");
                };
                shape_manager.on_create_mesh_edge(start, entity, end);
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_shape(ShapeType::Edge),
                )
            }
            ComponentData::MeshFace(project_key, file_key, vertex_a, vertex_b, vertex_c) => {
                let Some(shape_manager) = shape_manager_opt else {
                    panic!("shape manager not available");
                };
                let file_entity = git_manager.file_entity(&project_key, &file_key).unwrap();
                shape_manager.on_create_face(
                    &file_entity,
                    None,
                    entity,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                );
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_shape(ShapeType::Face),
                )
            }
            ComponentData::IconVertex(project_key, file_key) => {
                let Some(icon_manager) = icon_manager_opt else {
                    panic!("icon manager not available");
                };
                icon_manager.on_create_vertex(entity);
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_icon_shape(ShapeType::Vertex),
                )
            }
            ComponentData::IconEdge(project_key, file_key, start, end) => {
                let Some(icon_manager) = icon_manager_opt else {
                    panic!("icon manager not available");
                };
                icon_manager.on_create_edge(start, entity, end);
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_icon_shape(ShapeType::Edge),
                )
            }
            ComponentData::IconFace(
                project_key,
                file_key,
                frame_entity,
                vertex_a,
                vertex_b,
                vertex_c,
            ) => {
                let Some(icon_manager) = icon_manager_opt else {
                    panic!("icon manager not available");
                };
                icon_manager.on_create_face(
                    &frame_entity,
                    None,
                    entity,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                );
                (
                    project_key,
                    file_key,
                    ContentEntityData::new_icon_face(None),
                )
            }
            ComponentData::ModelTransform(project_key, file_key) => (
                project_key,
                file_key,
                ContentEntityData::new_net_transform(),
            ),
            ComponentData::SceneTransform(project_key, file_key) => (
                project_key,
                file_key,
                ContentEntityData::new_net_transform(),
            ),
        };

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
                    shape_manager_opt,
                    icon_manager_opt,
                    child_entity,
                    child_entry,
                );
            }
        }
    }

    fn contains_key(&self, entity: &Entity) -> bool {
        self.incomplete_entries.contains_key(entity)
    }

    fn insert_incomplete(&mut self, entity: Entity, entry: ComponentWaitlistEntry) {
        self.incomplete_entries.insert(entity, entry);
    }

    fn get_mut(&mut self, entity: &Entity) -> Option<&mut ComponentWaitlistEntry> {
        self.incomplete_entries.get_mut(entity)
    }

    fn remove(&mut self, entity: &Entity) -> Option<ComponentWaitlistEntry> {
        self.incomplete_entries.remove(entity)
    }
}
