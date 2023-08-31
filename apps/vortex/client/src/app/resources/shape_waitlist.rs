use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
};
use bevy_log::info;
use math::Vec3;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};
use vortex_proto::{components::FileTypeValue, resources::DependencyMap};

use crate::app::{
    components::{OwnedByFileLocal, Vertex2d},
    resources::{
        camera_manager::CameraManager, shape_manager::FaceKey, shape_manager::ShapeManager,
    },
};

pub enum ShapeWaitlistInsert {
    Vertex,
    VertexRoot,
    Edge(Entity, Entity),
    Face(Entity, Entity, Entity, Entity, Entity, Entity),
    OwnedByFile(Entity),
    FileType(FileTypeValue),
}

#[derive(Clone, Copy)]
enum ShapeType {
    Vertex,
    Edge,
    Face,
}

enum ShapeData {
    Vertex(Option<Entity>),
    Edge(Entity, Entity),
    Face(Entity, Entity, Entity, Entity, Entity, Entity),
}

#[derive(Clone)]
pub struct ShapeWaitlistEntry {
    shape: Option<ShapeType>,
    vertex_parent: Option<Option<Entity>>,
    file_entity: Option<Entity>,
    edge_entities: Option<(Entity, Entity)>,
    // Option<vertex a, vertex b, vertex c, edge a, edge b, edge c>
    face_entities: Option<(Entity, Entity, Entity, Entity, Entity, Entity)>,
    file_type: Option<FileTypeValue>,
}

impl ShapeWaitlistEntry {
    fn new() -> Self {
        Self {
            shape: None,
            vertex_parent: None,
            file_entity: None,
            edge_entities: None,
            face_entities: None,
            file_type: None,
        }
    }

    fn is_ready(&self) -> bool {
        match self.shape {
            Some(ShapeType::Vertex) => match self.file_type {
                None => return false,
                Some(FileTypeValue::Skel) => {
                    return self.file_entity.is_some() && self.vertex_parent.is_some()
                }
                Some(FileTypeValue::Mesh) => return self.file_entity.is_some(),
            },
            Some(ShapeType::Edge) => {
                self.file_type.is_some()
                    && self.file_entity.is_some()
                    && self.edge_entities.is_some()
            }
            Some(ShapeType::Face) => {
                self.file_type.is_some()
                    && self.file_entity.is_some()
                    && self.face_entities.is_some()
            }
            None => false,
        }
    }

    fn set_parent(&mut self, parent: Option<Entity>) {
        self.shape = Some(ShapeType::Vertex);
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
        self.shape = Some(ShapeType::Vertex);
    }

    fn set_edge(&mut self, start: Entity, end: Entity) {
        self.shape = Some(ShapeType::Edge);
        self.edge_entities = Some((start, end));
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
        self.shape = Some(ShapeType::Face);
        self.face_entities = Some((vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c));
        self.file_type = Some(FileTypeValue::Mesh);
    }

    fn set_file_entity(&mut self, file_entity: Entity) {
        self.file_entity = Some(file_entity);
    }

    fn set_file_type(&mut self, file_type: FileTypeValue) {
        self.file_type = Some(file_type);
    }

    fn decompose(self) -> (ShapeData, Entity, FileTypeValue) {
        let shape = self.shape.unwrap();
        let file_type = self.file_type.unwrap();

        let shape_data = match (shape, file_type) {
            (ShapeType::Vertex, FileTypeValue::Skel) => {
                ShapeData::Vertex(self.vertex_parent.unwrap())
            }
            (ShapeType::Vertex, FileTypeValue::Mesh) => ShapeData::Vertex(None),
            (ShapeType::Edge, _) => {
                let entities = self.edge_entities.unwrap();
                ShapeData::Edge(entities.0, entities.1)
            }
            (ShapeType::Face, _) => {
                let (vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) =
                    self.face_entities.unwrap();
                ShapeData::Face(vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c)
            }
        };
        return (shape_data, self.file_entity.unwrap(), file_type);
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
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        transform_q: &Query<&Transform>,
        entity: &Entity,
        insert: ShapeWaitlistInsert,
    ) {
        if !self.contains_key(&entity) {
            self.insert_incomplete(*entity, ShapeWaitlistEntry::new());
        }

        let mut possibly_ready_entities = vec![*entity];

        match insert {
            ShapeWaitlistInsert::Vertex => {
                self.get_mut(&entity).unwrap().set_vertex();
            }
            ShapeWaitlistInsert::VertexRoot => {
                self.get_mut(&entity).unwrap().set_parent(None);
            }
            ShapeWaitlistInsert::Edge(start_entity, end_entity) => {
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

                info!(
                    "Entities to check for readiness... `{:?}`",
                    possibly_ready_entities
                );
            }
            ShapeWaitlistInsert::Face(vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) => {
                self.get_mut(&entity)
                    .unwrap()
                    .set_face(vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c);
            }
            ShapeWaitlistInsert::OwnedByFile(file_entity) => {
                self.get_mut(&entity).unwrap().set_file_entity(file_entity);

                // insert local version of OwnedByFile
                commands
                    .entity(*entity)
                    .insert(OwnedByFileLocal::new(file_entity));
            }
            ShapeWaitlistInsert::FileType(file_type) => {
                self.get_mut(&entity).unwrap().set_file_type(file_type);
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
            info!("entity `{:?}` is ready!", entity);

            let entry = self.remove(&entity).unwrap();
            let entry_shape = entry.shape.unwrap();
            let entry_file_type = entry.file_type.unwrap();

            match (entry_shape, entry_file_type) {
                (ShapeType::Vertex, FileTypeValue::Skel) => {
                    if entry.has_parent() {
                        let parent_entity = entry.get_parent().unwrap();
                        if !shape_manager.has_vertex_entity_3d(&parent_entity) {
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
                (ShapeType::Edge, _) => {
                    let entities = entry.edge_entities.unwrap();

                    let mut dependencies = Vec::new();
                    for vertex_3d_entity in [&entities.0, &entities.1] {
                        if !shape_manager.has_vertex_entity_3d(vertex_3d_entity) {
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
                (ShapeType::Vertex, FileTypeValue::Mesh) => {}
                (ShapeType::Face, _) => {
                    let entities = entry.face_entities.unwrap();

                    let mut dependencies = Vec::new();

                    for vertex_3d_entity in [&entities.0, &entities.1, &entities.2] {
                        if !shape_manager.has_vertex_entity_3d(vertex_3d_entity) {
                            // need to put in parent waitlist
                            info!(
                                "face entity {:?} requires parent vertex {:?}. putting in parent waitlist",
                                entity, vertex_3d_entity
                            );
                            dependencies.push(*vertex_3d_entity);
                        }
                    }

                    for edge_3d_entity in [&entities.3, &entities.4, &entities.5] {
                        if !shape_manager.has_edge_entity_3d(edge_3d_entity) {
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
            }
            self.process_complete(
                commands,
                meshes,
                materials,
                camera_manager,
                shape_manager,
                transform_q,
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
        shape_manager: &mut ShapeManager,
        transform_q: &Query<&Transform>,
        entity: Entity,
        entry: ShapeWaitlistEntry,
    ) {
        let (shape_data, file_entity, file_type) = entry.decompose();

        match (shape_data, file_type) {
            (ShapeData::Vertex(parent_3d_entity_opt), FileTypeValue::Skel) => {
                let color = match parent_3d_entity_opt {
                    Some(_) => Vertex2d::CHILD_COLOR,
                    None => Vertex2d::ROOT_COLOR,
                };

                shape_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    parent_3d_entity_opt.is_none(),
                    Some(file_entity),
                    color,
                );
            }
            (ShapeData::Vertex(_), FileTypeValue::Mesh) => {
                let color = Vertex2d::CHILD_COLOR;

                shape_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    false,
                    Some(file_entity),
                    color,
                );
            }
            (ShapeData::Edge(start_3d, end_3d), _) => {
                let start_2d = shape_manager.vertex_entity_3d_to_2d(&start_3d).unwrap();
                let end_2d = shape_manager.vertex_entity_3d_to_2d(&end_3d).unwrap();

                shape_manager.edge_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    start_2d,
                    start_3d,
                    end_2d,
                    end_3d,
                    Some(file_entity),
                    Vertex2d::CHILD_COLOR,
                    file_type == FileTypeValue::Skel,
                );
            }
            (ShapeData::Face(vertex_a, vertex_b, vertex_c, _edge_a, _edge_b, _edge_c), _) => {
                let face_key = FaceKey::new(vertex_a, vertex_b, vertex_c);
                let mut positions = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
                for (index, vertex_3d_entity) in [vertex_a, vertex_b, vertex_c].iter().enumerate() {
                    let transform = transform_q.get(*vertex_3d_entity).unwrap();
                    positions[index] = transform.translation;
                }

                shape_manager.remove_new_face_key(&face_key);
                shape_manager.process_new_face(
                    commands,
                    camera_manager,
                    meshes,
                    materials,
                    file_entity,
                    &face_key,
                );
                shape_manager.face_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    &face_key,
                    entity,
                    positions,
                );
            }
        }

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
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    shape_manager,
                    transform_q,
                    child_entity,
                    child_entry,
                );
            }
        }

        camera_manager.recalculate_3d_view();
        shape_manager.recalculate_shapes();
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
