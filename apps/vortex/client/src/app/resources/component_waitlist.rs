use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use math::Vec3;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    Assets,
};

use vortex_proto::{
    components::{NetTransformEntityType, FileExtension, Vertex3d},
    resources::DependencyMap,
};

use crate::app::{
    components::{OwnedByFileLocal, Vertex2d},
    events::ShapeColorResyncEvent,
    resources::{icon_manager::IconManager,
        camera_manager::CameraManager, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, shape_data::FaceKey, vertex_manager::VertexManager, model_manager::ModelManager
    },
};

pub enum ComponentWaitlistInsert {
    FileType(FileExtension),
    OwnedByFile(Entity),
    Vertex,
    VertexRoot,
    Edge(Entity, Entity),
    Face(Entity, Entity, Entity, Entity, Entity, Entity),
    EdgeAngle(f32),
    NetTransform,
    SkinOrSceneEntity(Entity, NetTransformEntityType),
    ShapeName(String),
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
    Face(Entity, Entity, Entity, Entity, Entity, Entity),
    // SkinOrSceneEntity, Option<ShapeName>
    NetTransform(Option<String>),
}

impl ComponentData {
    pub fn to_type(&self) -> ComponentType {
        match self {
            ComponentData::Vertex(_) => ComponentType::Vertex,
            ComponentData::Edge(_, _, _) => ComponentType::Edge,
            ComponentData::Face(_, _, _, _, _, _) => ComponentType::Face,
            ComponentData::NetTransform(_) => ComponentType::NetTransform,
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
        }
    }

    fn is_ready(&self) -> bool {
        match self.component_type {
            Some(ComponentType::Vertex) => match self.file_type {
                None => return false,
                Some(FileExtension::Skel) => {
                    return self.file_entity.is_some() && self.vertex_parent.is_some()
                }
                Some(FileExtension::Mesh) => return self.file_entity.is_some(),
                Some(_) => {
                    panic!("");
                }
            },
            Some(ComponentType::Edge) => match self.file_type {
                None => return false,
                Some(FileExtension::Skel) => {
                    return self.file_entity.is_some()
                        && self.edge_entities.is_some()
                        && self.edge_angle.is_some()
                }
                Some(FileExtension::Mesh) => {
                    return self.file_entity.is_some() && self.edge_entities.is_some()
                }
                Some(_) => {
                    panic!("");
                }
            },
            Some(ComponentType::Face) => {
                self.file_type.is_some()
                    && self.file_entity.is_some()
                    && self.face_entities.is_some()
            }
            Some(ComponentType::NetTransform) => match self.file_type {
                None => return false,
                Some(FileExtension::Model) => {
                    return self.file_entity.is_some() && self.skin_or_scene_entity.is_some() && self.shape_name.is_some()
                }
                Some(FileExtension::Scene) => {
                    return self.file_entity.is_some() && self.skin_or_scene_entity.is_some()
                }
                Some(_) => {
                    panic!("");
                }
            }

            None => false,
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
        self.file_type = Some(FileExtension::Mesh);
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

    fn decompose(self) -> (ComponentData, Entity, FileExtension) {
        let shape = self.component_type.unwrap();
        let file_type = self.file_type.unwrap();

        let shape_data = match (shape, file_type) {
            (ComponentType::Vertex, FileExtension::Skel) => {
                ComponentData::Vertex(self.vertex_parent.unwrap())
            }
            (ComponentType::Vertex, FileExtension::Mesh) => ComponentData::Vertex(None),
            (ComponentType::Edge, FileExtension::Mesh) => {
                let entities = self.edge_entities.unwrap();
                ComponentData::Edge(entities.0, entities.1, None)
            }
            (ComponentType::Edge, FileExtension::Skel) => {
                let entities = self.edge_entities.unwrap();
                let edge_angle = self.edge_angle.unwrap();
                ComponentData::Edge(entities.0, entities.1, Some(edge_angle))
            }
            (ComponentType::Face, _) => {
                let (vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) =
                    self.face_entities.unwrap();
                ComponentData::Face(vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c)
            }
            (ComponentType::NetTransform, FileExtension::Model) => {
                ComponentData::NetTransform(Some(self.shape_name.unwrap()))
            }
            (ComponentType::NetTransform, FileExtension::Scene) => {
                ComponentData::NetTransform(None)
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
    pub fn process_insert(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        canvas: &mut Canvas,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        model_manager_opt: &mut Option<&mut ModelManager>,
        icon_manager: &mut Option<&mut IconManager>,
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
        vertex_3d_q: &Query<&Vertex3d>,
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
            ComponentWaitlistInsert::SkinOrSceneEntity(skin_or_scene_entity, skin_or_scene_type) => {
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
            // info!("entity `{:?}` is ready!", entity);

            let entry = self.remove(&entity).unwrap();
            let entry_shape = entry.component_type.unwrap();
            let entry_file_type = entry.file_type.unwrap();

            match (entry_shape, entry_file_type) {
                (ComponentType::Vertex, FileExtension::Skel) => {
                    if entry.has_parent() {
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
                (ComponentType::Edge, FileExtension::Skel | FileExtension::Mesh) => {
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
                (ComponentType::Face, FileExtension::Mesh) => {
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
                (ComponentType::Edge, FileExtension::Icon) => {
                    let entities = entry.edge_entities.unwrap();
                    let Some(icon_manager) = icon_manager else {
                        panic!("hmm");
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
                (ComponentType::Face, FileExtension::Icon) => {
                    let entities = entry.face_entities.unwrap();
                    let Some(icon_manager) = icon_manager else {
                        panic!("hmm");
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
                (ComponentType::Vertex, FileExtension::Mesh) | (ComponentType::NetTransform, FileExtension::Model | FileExtension::Scene) => {
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
                vertex_manager,
                edge_manager,
                face_manager,
                model_manager_opt,
                shape_color_resync_events,
                vertex_3d_q,
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
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        model_manager_opt: &mut Option<&mut ModelManager>,
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
        vertex_3d_q: &Query<&Vertex3d>,
        entity: Entity,
        entry: ComponentWaitlistEntry,
    ) {
        let (shape_data, file_entity, file_type) = entry.decompose();

        let shape_type = shape_data.to_type();

        match (file_type, shape_data) {
            (FileExtension::Skel, ComponentData::Vertex(parent_3d_entity_opt)) => {
                let color = match parent_3d_entity_opt {
                    Some(_) => Vertex2d::ENABLED_COLOR,
                    None => Vertex2d::ROOT_COLOR,
                };

                vertex_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    parent_3d_entity_opt,
                    parent_3d_entity_opt.is_none(),
                    Some(file_entity),
                    color,
                    false,
                );
            }
            (FileExtension::Mesh, ComponentData::Vertex(_)) => {
                let color = Vertex2d::ENABLED_COLOR;

                vertex_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    None,
                    false,
                    Some(file_entity),
                    color,
                    true,
                );
            }
            (FileExtension::Skel, ComponentData::Edge(start_3d, end_3d, edge_angle_opt)) => {
                let start_2d = vertex_manager.vertex_entity_3d_to_2d(&start_3d).unwrap();
                let end_2d = vertex_manager.vertex_entity_3d_to_2d(&end_3d).unwrap();

                edge_manager.edge_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    vertex_manager,
                    None,
                    None,
                    entity,
                    start_2d,
                    start_3d,
                    end_2d,
                    end_3d,
                    Some(file_entity),
                    Vertex2d::ENABLED_COLOR,
                    true,
                    edge_angle_opt,
                    false,
                );
            }
            (FileExtension::Mesh, ComponentData::Edge(start_3d, end_3d, None)) => {
                let start_2d = vertex_manager.vertex_entity_3d_to_2d(&start_3d).unwrap();
                let end_2d = vertex_manager.vertex_entity_3d_to_2d(&end_3d).unwrap();

                edge_manager.edge_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    vertex_manager,
                    Some(face_manager),
                    Some(shape_color_resync_events),
                    entity,
                    start_2d,
                    start_3d,
                    end_2d,
                    end_3d,
                    Some(file_entity),
                    Vertex2d::ENABLED_COLOR,
                    false,
                    None,
                    true,
                );
            }
            (_, ComponentData::Face(vertex_a, vertex_b, vertex_c, _edge_a, _edge_b, _edge_c)) => {
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
            (FileExtension::Model, ComponentData::NetTransform(Some(shape_name))) => {

                model_manager_opt.as_mut().unwrap().net_transform_postprocess(
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

                model_manager_opt.as_mut().unwrap().net_transform_postprocess(
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
                    vertex_manager,
                    edge_manager,
                    face_manager,
                    model_manager_opt,
                    shape_color_resync_events,
                    vertex_3d_q,
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
