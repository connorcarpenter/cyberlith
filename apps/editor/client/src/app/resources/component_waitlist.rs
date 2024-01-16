use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use math::{Vec2, Vec3};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    Assets,
};

use editor_proto::{
    components::{FileExtension, IconVertex, NetTransformEntityType, Vertex3d},
    resources::DependencyMap,
};

use crate::app::{
    components::{OwnedByFileLocal, Vertex2d},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, icon_data::IconFaceKey, icon_manager::IconManager,
        model_manager::ModelManager, shape_data::FaceKey, vertex_manager::VertexManager,
    },
};

#[derive(Clone)]
pub enum ComponentWaitlistInsert {
    FileType(FileExtension),
    OwnedByFile(Entity),
    Vertex,
    VertexRoot,
    Edge(Entity, Entity),
    // vertex_entity_a, vertex_entity_b, vertex_entity_c, edge_entity_a, edge_entity_b, edge_entity_c
    Face(Entity, Entity, Entity, Entity, Entity, Entity),
    // color_entity, vertex_entity_a, vertex_entity_b, vertex_entity_c, edge_entity_a, edge_entity_b, edge_entity_c
    IconFace(Entity, Entity, Entity, Entity, Entity, Entity, Entity),
    EdgeAngle(f32),
    NetTransform,
    SkinOrSceneEntity(Entity, NetTransformEntityType),
    ShapeName(String),
    FrameEntity(Entity),
}

#[derive(Clone, Copy, Debug)]
enum ComponentType {
    Vertex,
    Edge,
    Face,
    NetTransform,
}

enum ComponentData {
    //parent_3d_entity_opt
    Vertex(Option<Entity>),
    Edge(Entity, Entity, Option<f32>),
    // vertex a, vertex b, vertex c
    Face(Entity, Entity, Entity),
    // SkinOrSceneEntity, Option<ShapeName>
    NetTransform(Option<String>),
    IconVertex(Entity),
    IconEdge(Entity, Entity, Entity),
    // frame entity, vertex a, vertex b, vertex c
    IconFace(Entity, Entity, Entity, Entity),
}

impl ComponentData {
    pub fn to_type(&self) -> ComponentType {
        match self {
            ComponentData::Vertex(_) => ComponentType::Vertex,
            ComponentData::Edge(_, _, _) => ComponentType::Edge,
            ComponentData::Face(_, _, _) => ComponentType::Face,
            ComponentData::NetTransform(_) => ComponentType::NetTransform,
            ComponentData::IconVertex(_) => ComponentType::Vertex,
            ComponentData::IconEdge(_, _, _) => ComponentType::Edge,
            ComponentData::IconFace(_, _, _, _) => ComponentType::Face,
        }
    }
}

#[derive(Clone)]
pub struct ComponentWaitlistEntry {
    component_type: Option<ComponentType>,
    file_type: Option<FileExtension>,
    file_entity: Option<Entity>,

    vertex_parent: Option<Option<Entity>>,
    edge_entities: Option<(Entity, Entity)>,
    edge_angle: Option<f32>,
    // Option<vertex a, vertex b, vertex c, edge a, edge b, edge c>
    face_entities: Option<(Entity, Entity, Entity, Entity, Entity, Entity)>,

    skin_or_scene_entity: Option<(Entity, NetTransformEntityType)>,
    shape_name: Option<String>,
    icon_frame_entity: Option<Entity>,
    icon_color_entity: Option<Entity>,
}

impl ComponentWaitlistEntry {
    fn new() -> Self {
        Self {
            component_type: None,
            vertex_parent: None,
            file_entity: None,
            edge_entities: None,
            face_entities: None,
            file_type: None,
            edge_angle: None,
            skin_or_scene_entity: None,
            shape_name: None,
            icon_frame_entity: None,
            icon_color_entity: None,
        }
    }

    fn is_ready(&self) -> bool {
        match (self.file_type, self.component_type) {
            (Some(FileExtension::Skel), Some(ComponentType::Vertex)) => {
                return self.file_entity.is_some() && self.vertex_parent.is_some();
            }
            (Some(FileExtension::Skel), Some(ComponentType::Edge)) => {
                return self.file_entity.is_some()
                    && self.edge_entities.is_some()
                    && self.edge_angle.is_some();
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Vertex)) => {
                return self.file_entity.is_some();
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Edge)) => {
                return self.file_entity.is_some() && self.edge_entities.is_some();
            }
            (Some(FileExtension::Mesh), Some(ComponentType::Face)) => {
                self.file_entity.is_some() && self.face_entities.is_some()
            }
            (Some(FileExtension::Icon), Some(ComponentType::Vertex)) => {
                return self.file_entity.is_some() && self.icon_frame_entity.is_some();
            }
            (Some(FileExtension::Icon), Some(ComponentType::Edge)) => {
                return self.file_entity.is_some()
                    && self.edge_entities.is_some()
                    && self.icon_frame_entity.is_some();
            }
            (Some(FileExtension::Icon), Some(ComponentType::Face)) => {
                info!("checking icon face. file_entity: {:?}, face_entities: {:?}, icon_frame_entity: {:?}, icon_color_entity: {:?}",
                    self.file_entity,
                    self.face_entities,
                    self.icon_frame_entity,
                    self.icon_color_entity,
                );
                self.file_entity.is_some()
                    && self.face_entities.is_some()
                    && self.icon_frame_entity.is_some()
                    && self.icon_color_entity.is_some()
            }
            (Some(FileExtension::Model), Some(ComponentType::NetTransform)) => {
                return self.file_entity.is_some()
                    && self.skin_or_scene_entity.is_some()
                    && self.shape_name.is_some();
            }
            (Some(FileExtension::Scene), Some(ComponentType::NetTransform)) => {
                return self.file_entity.is_some() && self.skin_or_scene_entity.is_some();
            }
            (_, _) => false,
        }
    }

    fn set_file_entity(&mut self, file_entity: Entity) {
        self.file_entity = Some(file_entity);
    }

    fn set_file_type(&mut self, file_type: FileExtension) {
        self.file_type = Some(file_type);
    }

    fn set_parent(&mut self, parent: Option<Entity>) {
        self.component_type = Some(ComponentType::Vertex);
        self.vertex_parent = Some(parent);
    }

    fn get_parent(&self) -> Option<Entity> {
        self.vertex_parent.unwrap()
    }

    fn has_parent(&self) -> bool {
        if let Some(parent_opt) = &self.vertex_parent {
            return parent_opt.is_some();
        }
        return false;
    }

    fn set_vertex(&mut self) {
        self.component_type = Some(ComponentType::Vertex);
    }

    fn set_edge(&mut self, start: Entity, end: Entity) {
        self.component_type = Some(ComponentType::Edge);
        self.edge_entities = Some((start, end));
    }

    fn set_edge_angle(&mut self, angle: f32) {
        self.edge_angle = Some(angle);
    }

    fn set_face(
        &mut self,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
        edge_a: Entity,
        edge_b: Entity,
        edge_c: Entity,
    ) {
        self.component_type = Some(ComponentType::Face);
        self.face_entities = Some((vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c));
    }

    fn set_icon_face(
        &mut self,
        color_entity: Entity,
        vertex_a: Entity,
        vertex_b: Entity,
        vertex_c: Entity,
        edge_a: Entity,
        edge_b: Entity,
        edge_c: Entity,
    ) {
        self.component_type = Some(ComponentType::Face);
        self.face_entities = Some((vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c));
        self.icon_color_entity = Some(color_entity);
    }

    fn set_transform(&mut self) {
        self.component_type = Some(ComponentType::NetTransform);
    }

    fn set_skin_or_scene_entity(&mut self, entity: Entity, entity_type: NetTransformEntityType) {
        self.skin_or_scene_entity = Some((entity, entity_type));
    }

    fn set_shape_name(&mut self, shape_name: String) {
        self.shape_name = Some(shape_name);
    }

    fn set_frame_entity(&mut self, frame_entity: Entity) {
        self.icon_frame_entity = Some(frame_entity);
    }

    fn decompose(self) -> (ComponentData, Entity, FileExtension) {
        let shape = self.component_type.unwrap();
        let file_type = self.file_type.unwrap();

        let shape_data = match (file_type, shape) {
            (FileExtension::Skel, ComponentType::Vertex) => {
                ComponentData::Vertex(self.vertex_parent.unwrap())
            }
            (FileExtension::Skel, ComponentType::Edge) => {
                let entities = self.edge_entities.unwrap();
                let edge_angle = self.edge_angle.unwrap();
                ComponentData::Edge(entities.0, entities.1, Some(edge_angle))
            }
            (FileExtension::Mesh, ComponentType::Vertex) => ComponentData::Vertex(None),
            (FileExtension::Mesh, ComponentType::Edge) => {
                let entities = self.edge_entities.unwrap();
                ComponentData::Edge(entities.0, entities.1, None)
            }
            (FileExtension::Mesh, ComponentType::Face) => {
                let (vertex_a, vertex_b, vertex_c, _, _, _) = self.face_entities.unwrap();
                ComponentData::Face(vertex_a, vertex_b, vertex_c)
            }
            (FileExtension::Model, ComponentType::NetTransform) => {
                ComponentData::NetTransform(Some(self.shape_name.unwrap()))
            }
            (FileExtension::Scene, ComponentType::NetTransform) => {
                ComponentData::NetTransform(None)
            }
            (FileExtension::Icon, ComponentType::Vertex) => {
                ComponentData::IconVertex(self.icon_frame_entity.unwrap())
            }
            (FileExtension::Icon, ComponentType::Edge) => {
                let entities = self.edge_entities.unwrap();
                ComponentData::IconEdge(self.icon_frame_entity.unwrap(), entities.0, entities.1)
            }
            (FileExtension::Icon, ComponentType::Face) => {
                let (vertex_a, vertex_b, vertex_c, _, _, _) = self.face_entities.unwrap();
                ComponentData::IconFace(
                    self.icon_frame_entity.unwrap(),
                    vertex_a,
                    vertex_b,
                    vertex_c,
                )
            }
            (_, _) => {
                panic!("");
            }
        };
        return (shape_data, self.file_entity.unwrap(), file_type);
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
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        canvas: &mut Canvas,
        vertex_manager_opt: &mut Option<&mut VertexManager>,
        edge_manager_opt: &mut Option<&mut EdgeManager>,
        face_manager_opt: &mut Option<&mut FaceManager>,
        model_manager_opt: &mut Option<&mut ModelManager>,
        icon_manager_opt: &mut Option<&mut IconManager>,
        vertex_3d_q_opt: Option<&Query<&Vertex3d>>,
        icon_vertex_q_opt: Option<&Query<&IconVertex>>,
        entity: &Entity,
        inserts: &[ComponentWaitlistInsert],
    ) {
        for insert in inserts {
            self.process_insert(
                commands,
                meshes,
                materials,
                camera_manager,
                canvas,
                vertex_manager_opt,
                edge_manager_opt,
                face_manager_opt,
                model_manager_opt,
                icon_manager_opt,
                vertex_3d_q_opt,
                icon_vertex_q_opt,
                entity,
                insert.clone(),
            );
        }
    }

    pub fn process_insert(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        canvas: &mut Canvas,
        vertex_manager_opt: &mut Option<&mut VertexManager>,
        edge_manager_opt: &mut Option<&mut EdgeManager>,
        face_manager_opt: &mut Option<&mut FaceManager>,
        model_manager_opt: &mut Option<&mut ModelManager>,
        icon_manager_opt: &mut Option<&mut IconManager>,
        vertex_3d_q_opt: Option<&Query<&Vertex3d>>,
        icon_vertex_q_opt: Option<&Query<&IconVertex>>,
        entity: &Entity,
        insert: ComponentWaitlistInsert,
    ) {
        if !self.contains_key(&entity) {
            self.insert_incomplete(*entity, ComponentWaitlistEntry::new());
        }

        let mut possibly_ready_entities = vec![*entity];

        match insert {
            ComponentWaitlistInsert::FileType(file_type) => {
                self.get_mut(&entity).unwrap().set_file_type(file_type);
            }
            ComponentWaitlistInsert::OwnedByFile(file_entity) => {
                self.get_mut(&entity).unwrap().set_file_entity(file_entity);

                // insert local version of OwnedByFile
                commands
                    .entity(*entity)
                    .insert(OwnedByFileLocal::new(file_entity));
            }
            ComponentWaitlistInsert::Vertex => {
                self.get_mut(&entity).unwrap().set_vertex();
            }
            ComponentWaitlistInsert::VertexRoot => {
                self.get_mut(&entity).unwrap().set_parent(None);
            }
            ComponentWaitlistInsert::Edge(start_entity, end_entity) => {
                let Some(edge_entry) = self.get_mut(&entity) else {
                    panic!("edge entity {:?} should have been inserted already!", entity);
                };
                edge_entry.set_edge(start_entity, end_entity);

                if let Some(vertex_entry) = self.get_mut(&end_entity) {
                    // skel vertices will wait around for a parent, and will be here
                    // mesh vertices should already be gone, so will skip this block, or set the vertices parent which will be overwritten later

                    info!("Setting parent of {:?} to {:?}", end_entity, start_entity);
                    vertex_entry.set_parent(Some(start_entity));
                    possibly_ready_entities.push(end_entity);
                };

                // info!(
                //     "Entities to check for readiness... `{:?}`",
                //     possibly_ready_entities
                // );
            }
            ComponentWaitlistInsert::EdgeAngle(angle) => {
                self.get_mut(&entity).unwrap().set_edge_angle(angle);
            }
            ComponentWaitlistInsert::Face(vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) => {
                self.get_mut(&entity)
                    .unwrap()
                    .set_face(vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c);
            }
            ComponentWaitlistInsert::IconFace(
                color_entity,
                vertex_a,
                vertex_b,
                vertex_c,
                edge_a,
                edge_b,
                edge_c,
            ) => {
                self.get_mut(&entity).unwrap().set_icon_face(
                    color_entity,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                    edge_a,
                    edge_b,
                    edge_c,
                );
            }
            ComponentWaitlistInsert::SkinOrSceneEntity(
                skin_or_scene_entity,
                skin_or_scene_type,
            ) => {
                self.get_mut(&entity)
                    .unwrap()
                    .set_skin_or_scene_entity(skin_or_scene_entity, skin_or_scene_type);
            }
            ComponentWaitlistInsert::ShapeName(shape_name) => {
                self.get_mut(&entity).unwrap().set_shape_name(shape_name);
            }
            ComponentWaitlistInsert::NetTransform => {
                self.get_mut(&entity).unwrap().set_transform();
            }
            ComponentWaitlistInsert::FrameEntity(frame_entity) => {
                self.get_mut(&entity)
                    .unwrap()
                    .set_frame_entity(frame_entity);
            }
        }

        for possibly_ready_entity in possibly_ready_entities {
            let Some(incomplete_entry) = self
                .incomplete_entries
                .get(&possibly_ready_entity) else {
                panic!("entity {:?} should have been inserted already!", possibly_ready_entity);
            };
            if !incomplete_entry.is_ready() {
                info!("entity `{:?}` is not ready yet...", possibly_ready_entity);
                return;
            }

            // entity is ready!
            let entity = possibly_ready_entity;

            let entry = self.remove(&entity).unwrap();
            let entry_shape = entry.component_type.unwrap();
            let entry_file_type = entry.file_type.unwrap();

            match (entry_file_type, entry_shape) {
                (FileExtension::Skel, ComponentType::Vertex) => {
                    if entry.has_parent() {
                        let Some(vertex_manager) = vertex_manager_opt else {
                            panic!("vertex manager not available");
                        };

                        let parent_entity = entry.get_parent().unwrap();
                        if !vertex_manager.has_vertex_entity_3d(&parent_entity) {
                            // need to put in parent waitlist
                            info!(
                                "vert entity {:?} requires parent {:?}. putting in parent waitlist",
                                entity, parent_entity
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
                (FileExtension::Skel | FileExtension::Mesh, ComponentType::Edge) => {
                    let Some(vertex_manager) = vertex_manager_opt else {
                        panic!("vertex manager not available");
                    };
                    let entities = entry.edge_entities.unwrap();

                    let mut dependencies = Vec::new();
                    for vertex_3d_entity in [&entities.0, &entities.1] {
                        if !vertex_manager.has_vertex_entity_3d(vertex_3d_entity) {
                            // need to put in parent waitlist
                            info!(
                                "edge entity {:?} requires parent {:?}. putting in parent waitlist",
                                entity, vertex_3d_entity
                            );
                            dependencies.push(*vertex_3d_entity);
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
                    let Some(vertex_manager) = vertex_manager_opt else {
                        panic!("vertex manager not available");
                    };
                    let Some(edge_manager) = edge_manager_opt else {
                        panic!("edge manager not available");
                    };
                    let entities = entry.face_entities.unwrap();

                    let mut dependencies = Vec::new();

                    for vertex_3d_entity in [&entities.0, &entities.1, &entities.2] {
                        if !vertex_manager.has_vertex_entity_3d(vertex_3d_entity) {
                            // need to put in parent waitlist
                            info!(
                                "face entity {:?} requires parent vertex {:?}. putting in parent waitlist",
                                entity, vertex_3d_entity
                            );
                            dependencies.push(*vertex_3d_entity);
                        }
                    }

                    for edge_3d_entity in [&entities.3, &entities.4, &entities.5] {
                        if !edge_manager.has_edge_entity_3d(edge_3d_entity) {
                            // need to put in parent waitlist
                            info!(
                                "face entity {:?} requires parent edge {:?}. putting in parent waitlist",
                                entity, edge_3d_entity
                            );
                            dependencies.push(*edge_3d_entity);
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
                (FileExtension::Icon, ComponentType::Edge) => {
                    let entities = entry.edge_entities.unwrap();
                    let Some(icon_manager) = icon_manager_opt else {
                        panic!("icon manager not available");
                    };

                    let mut dependencies = Vec::new();
                    for vertex_entity in [&entities.0, &entities.1] {
                        if !icon_manager.has_vertex_entity(vertex_entity) {
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
                    let entities = entry.face_entities.unwrap();
                    let Some(icon_manager) = icon_manager_opt else {
                        panic!("icon manager not available");
                    };

                    let mut dependencies = Vec::new();

                    for vertex_entity in [&entities.0, &entities.1, &entities.2] {
                        if !icon_manager.has_vertex_entity(vertex_entity) {
                            // need to put in parent waitlist
                            info!(
                                "face entity {:?} requires parent vertex {:?}. putting in parent waitlist",
                                entity, vertex_entity
                            );
                            dependencies.push(*vertex_entity);
                        }
                    }

                    for edge_entity in [&entities.3, &entities.4, &entities.5] {
                        if !icon_manager.has_edge_entity(edge_entity) {
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
                (FileExtension::Mesh | FileExtension::Icon, ComponentType::Vertex)
                | (FileExtension::Model | FileExtension::Scene, ComponentType::NetTransform) => {
                    // no dependencies
                }
                (_, _) => {
                    panic!("");
                }
            }
            self.process_complete(
                commands,
                meshes,
                materials,
                camera_manager,
                canvas,
                vertex_manager_opt,
                edge_manager_opt,
                face_manager_opt,
                model_manager_opt,
                icon_manager_opt,
                vertex_3d_q_opt,
                icon_vertex_q_opt,
                entity,
                entry,
            );
        }
    }

    fn process_complete(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        canvas: &mut Canvas,
        vertex_manager_opt: &mut Option<&mut VertexManager>,
        edge_manager_opt: &mut Option<&mut EdgeManager>,
        face_manager_opt: &mut Option<&mut FaceManager>,
        model_manager_opt: &mut Option<&mut ModelManager>,
        icon_manager_opt: &mut Option<&mut IconManager>,
        vertex_3d_q_opt: Option<&Query<&Vertex3d>>,
        icon_vertex_q_opt: Option<&Query<&IconVertex>>,
        entity: Entity,
        entry: ComponentWaitlistEntry,
    ) {
        let (shape_data, file_entity, file_type) = entry.decompose();

        let shape_type = shape_data.to_type();

        match (file_type, shape_data) {
            (FileExtension::Skel, ComponentData::Vertex(parent_3d_entity_opt)) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let color = match parent_3d_entity_opt {
                    Some(_) => Vertex2d::ENABLED_COLOR,
                    None => Vertex2d::ROOT_COLOR,
                };
                let mat_handle = materials.add(CpuMaterial::new(color, 0.0, 0.0, 0.0));
                vertex_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    &mat_handle,
                    camera_manager,
                    entity,
                    parent_3d_entity_opt,
                    parent_3d_entity_opt.is_none(),
                    Some(file_entity),
                    false,
                );
            }
            (FileExtension::Mesh, ComponentData::Vertex(_)) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let color = Vertex2d::ENABLED_COLOR;
                let mat_handle = materials.add(CpuMaterial::new(color, 0.0, 0.0, 0.0));

                vertex_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    &mat_handle,
                    camera_manager,
                    entity,
                    None,
                    false,
                    Some(file_entity),
                    true,
                );
            }
            (FileExtension::Skel, ComponentData::Edge(start_3d, end_3d, Some(edge_angle))) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let Some(edge_manager) = edge_manager_opt else {
                    panic!("edge manager not available");
                };
                let start_2d = vertex_manager.vertex_entity_3d_to_2d(&start_3d).unwrap();
                let end_2d = vertex_manager.vertex_entity_3d_to_2d(&end_3d).unwrap();

                let mat_handle = materials.add(CpuMaterial::new(Vertex2d::ENABLED_COLOR, 0.0, 0.0, 0.0));
                edge_manager.edge_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    &mat_handle,
                    camera_manager,
                    vertex_manager,
                    None,
                    entity,
                    start_2d,
                    start_3d,
                    end_2d,
                    end_3d,
                    Some(file_entity),
                    true,
                    Some(edge_angle),
                    false,
                );
            }
            (FileExtension::Mesh, ComponentData::Edge(start_3d, end_3d, None)) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let Some(edge_manager) = edge_manager_opt else {
                    panic!("edge manager not available");
                };
                let Some(face_manager) = face_manager_opt else {
                    panic!("face manager not available");
                };
                let start_2d = vertex_manager.vertex_entity_3d_to_2d(&start_3d).unwrap();
                let end_2d = vertex_manager.vertex_entity_3d_to_2d(&end_3d).unwrap();

                let mat_handle = materials.add(CpuMaterial::new(Vertex2d::ENABLED_COLOR, 0.0, 0.0, 0.0));
                edge_manager.edge_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    &mat_handle,
                    camera_manager,
                    vertex_manager,
                    Some(face_manager),
                    entity,
                    start_2d,
                    start_3d,
                    end_2d,
                    end_3d,
                    Some(file_entity),
                    false,
                    None,
                    true,
                );
            }
            (FileExtension::Mesh, ComponentData::Face(vertex_a, vertex_b, vertex_c)) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let Some(edge_manager) = edge_manager_opt else {
                    panic!("edge manager not available");
                };
                let Some(face_manager) = face_manager_opt else {
                    panic!("face manager not available");
                };
                let Some(vertex_3d_q) = vertex_3d_q_opt else {
                    panic!("vertex 3d q not available");
                };
                let face_key = FaceKey::new(vertex_a, vertex_b, vertex_c);
                let mut positions = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
                for (index, vertex_3d_entity) in [vertex_a, vertex_b, vertex_c].iter().enumerate() {
                    let vertex_3d = vertex_3d_q.get(*vertex_3d_entity).unwrap();
                    positions[index] = vertex_3d.as_vec3();
                }

                warn!("removing face key: {:?}", face_key);
                face_manager.remove_new_face_key(&face_key);
                if !face_manager.has_2d_face(&face_key) {
                    face_manager.process_new_face(
                        commands,
                        camera_manager,
                        vertex_manager,
                        edge_manager,
                        meshes,
                        materials,
                        file_entity,
                        &face_key,
                    );
                }
                face_manager.face_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    &face_key,
                    entity,
                    positions,
                );
            }
            (FileExtension::Icon, ComponentData::IconVertex(frame_entity)) => {
                let Some(icon_manager) = icon_manager_opt else {
                    panic!("icon manager not available");
                };

                let mat_handle = materials.add(CpuMaterial::new(Vertex2d::ENABLED_COLOR, 0.0, 0.0, 0.0));

                icon_manager.vertex_postprocess(
                    commands,
                    meshes,
                    &mat_handle,
                    Some(file_entity),
                    Some(frame_entity),
                    entity,
                );
            }
            (FileExtension::Icon, ComponentData::IconEdge(frame_entity, start, end)) => {
                let Some(icon_manager) = icon_manager_opt else {
                    panic!("icon manager not available");
                };

                let mat_handle = materials.add(CpuMaterial::new(Vertex2d::ENABLED_COLOR, 0.0, 0.0, 0.0));

                icon_manager.edge_postprocess(
                    commands,
                    meshes,
                    &mat_handle,
                    Some(file_entity),
                    Some(frame_entity),
                    entity,
                    start,
                    end,
                );
            }
            (
                FileExtension::Icon,
                ComponentData::IconFace(frame_entity, vertex_a, vertex_b, vertex_c),
            ) => {
                let Some(icon_manager) = icon_manager_opt else {
                    panic!("icon manager not available");
                };
                let Some(icon_vertex_q) = icon_vertex_q_opt else {
                    panic!("icon vertex q not available");
                };
                let face_key = IconFaceKey::new(vertex_a, vertex_b, vertex_c);
                let mut positions = [Vec2::ZERO, Vec2::ZERO, Vec2::ZERO];
                for (index, vertex_3d_entity) in [vertex_a, vertex_b, vertex_c].iter().enumerate() {
                    let icon_vertex = icon_vertex_q.get(*vertex_3d_entity).unwrap();
                    positions[index] = icon_vertex.as_vec2();
                }

                warn!("removing icon face key: {:?}", face_key);
                icon_manager.remove_new_face_key(&face_key);
                if !icon_manager.has_local_face(&face_key) {
                    icon_manager.process_new_local_face(
                        commands,
                        meshes,
                        materials,
                        &file_entity,
                        &frame_entity,
                        &face_key,
                    );
                }
                icon_manager.net_face_postprocess(
                    commands, meshes, materials, &face_key, entity, positions,
                );
            }
            (FileExtension::Model, ComponentData::NetTransform(Some(shape_name))) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let Some(edge_manager) = edge_manager_opt else {
                    panic!("edge manager not available");
                };

                model_manager_opt
                    .as_mut()
                    .unwrap()
                    .net_transform_postprocess(
                        commands,
                        camera_manager,
                        vertex_manager,
                        edge_manager,
                        meshes,
                        materials,
                        &file_entity,
                        Some(shape_name),
                        entity,
                    );
            }
            (FileExtension::Scene, ComponentData::NetTransform(None)) => {
                let Some(vertex_manager) = vertex_manager_opt else {
                    panic!("vertex manager not available");
                };
                let Some(edge_manager) = edge_manager_opt else {
                    panic!("edge manager not available");
                };

                model_manager_opt
                    .as_mut()
                    .unwrap()
                    .net_transform_postprocess(
                        commands,
                        camera_manager,
                        vertex_manager,
                        edge_manager,
                        meshes,
                        materials,
                        &file_entity,
                        None,
                        entity,
                    );
            }
            (_, _) => {
                panic!("");
            }
        }

        // if the waitlist has any children entities of this one, process them
        info!(
            "processing complete for shape `{:?}` of type: {:?}. checking for children..",
            entity, shape_type,
        );
        if let Some(child_entries) = self.dependency_map.on_dependency_complete(entity) {
            for (child_entity, child_entry) in child_entries {
                info!(
                    "entity {:?} was waiting on parent {:?}. processing!",
                    child_entity, entity
                );
                self.process_complete(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    canvas,
                    vertex_manager_opt,
                    edge_manager_opt,
                    face_manager_opt,
                    model_manager_opt,
                    icon_manager_opt,
                    vertex_3d_q_opt,
                    icon_vertex_q_opt,
                    child_entity,
                    child_entry,
                );
            }
        }

        canvas.queue_resync_shapes();
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
