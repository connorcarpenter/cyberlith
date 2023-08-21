use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource},
};
use bevy_log::info;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    Assets,
};
use vortex_proto::components::FileTypeValue;

use vortex_proto::types::TabId;

use crate::app::{
    components::{Edge3dLocal, Vertex2d},
    resources::{camera_manager::CameraManager, shape_manager::ShapeManager},
};

pub enum ShapeWaitlistInsert {
    Vertex,
    VertexRoot,
    Edge(Entity, Entity),
    OwnedByTab(TabId),
    FileType(FileTypeValue),
}

#[derive(Clone, Copy)]
enum ShapeType {
    Vertex,
    Edge,
}

enum ShapeData {
    Vertex(Option<Entity>),
    Edge(Entity, Entity),
}

#[derive(Clone)]
pub struct ShapeWaitlistEntry {
    shape: Option<ShapeType>,
    vertex_parent: Option<Option<Entity>>,
    tab_id: Option<TabId>,
    edge_entities: Option<(Entity, Entity)>,
    file_type: Option<FileTypeValue>,
}

impl ShapeWaitlistEntry {
    fn new() -> Self {
        Self {
            shape: None,
            vertex_parent: None,
            tab_id: None,
            edge_entities: None,
            file_type: None,
        }
    }

    fn is_ready(&self) -> bool {
        match self.shape {
            Some(ShapeType::Vertex) => {
                match self.file_type {
                    None => return false,
                    Some(FileTypeValue::Skel) => return self.tab_id.is_some() && self.vertex_parent.is_some(),
                    Some(FileTypeValue::Mesh) => return self.tab_id.is_some(),
                }

            },
            Some(ShapeType::Edge) =>   self.file_type.is_some() && self.tab_id.is_some() && self.edge_entities.is_some(),
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

    fn set_tab_id(&mut self, tab_id: TabId) {
        self.tab_id = Some(tab_id);
    }

    fn set_file_type(&mut self, file_type: FileTypeValue) {
        self.file_type = Some(file_type);
    }

    fn decompose(self) -> (ShapeData, TabId, FileTypeValue) {
        let shape = self.shape.unwrap();
        let file_type = self.file_type.unwrap();

        let shape_data = match (shape, file_type) {
            (ShapeType::Vertex, FileTypeValue::Skel) => ShapeData::Vertex(self.vertex_parent.unwrap()),
            (ShapeType::Vertex, FileTypeValue::Mesh) => ShapeData::Vertex(None),
            (ShapeType::Edge, _) => {
                let entities = self.edge_entities.unwrap();
                ShapeData::Edge(entities.0, entities.1)
            }
        };
        return (shape_data, self.tab_id.unwrap(), file_type);
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
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
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
            ShapeWaitlistInsert::VertexRoot => {
                self.get_mut(&entity).unwrap().set_parent(None);
            }
            ShapeWaitlistInsert::OwnedByTab(tab_id) => {
                self.get_mut(&entity).unwrap().set_tab_id(tab_id);
            }
            ShapeWaitlistInsert::FileType(file_type) => {
                info!("entity {:?} is set as file type: {:?}", entity, file_type);
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
                                entity,
                                parent_entity
                            );
                            self.insert_waiting_dependency(parent_entity, entity, entry);
                            continue;
                        }
                    }
                    info!("processing vertex {:?}", entity);
                }
                (ShapeType::Vertex, FileTypeValue::Mesh) => {
                    info!("processing vertex {:?}", entity);
                }
                (ShapeType::Edge, _) => {
                    let entities = entry.edge_entities.unwrap();
                    let mut has_all_entities = true;
                    if !shape_manager.has_vertex_entity_3d(&entities.0) {
                        // need to put in parent waitlist
                        info!(
                            "edge entity {:?} requires parent {:?}. putting in parent waitlist",
                            entity, entities.0
                        );
                        self.insert_waiting_dependency(entities.0, entity, entry.clone());
                        has_all_entities = false;
                    }
                    if !shape_manager.has_vertex_entity_3d(&entities.1) {
                        // need to put in parent waitlist
                        info!(
                            "edge entity {:?} requires parent {:?}. putting in parent waitlist",
                            entity, entities.1
                        );
                        self.insert_waiting_dependency(entities.1, entity, entry.clone());
                        has_all_entities = false;
                    }
                    if !has_all_entities {
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
        entity: Entity,
        entry: ShapeWaitlistEntry,
    ) {
        // info!("processing complete vertex {:?}", entity);

        let (shape_data, tab_id, file_type) = entry.decompose();

        match (shape_data, file_type) {
            (ShapeData::Vertex(parent_3d_entity_opt), FileTypeValue::Skel) => {
                let color = match parent_3d_entity_opt {
                    Some(_) => Vertex2d::CHILD_COLOR,
                    None => Vertex2d::ROOT_COLOR,
                };

                let _new_vertex_2d_entity = shape_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    parent_3d_entity_opt.is_none(),
                    Some(tab_id),
                    color,
                );

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
                            commands,
                            meshes,
                            materials,
                            camera_manager,
                            shape_manager,
                            child_entity,
                            child_entry,
                        );
                    }
                }
            }
            (ShapeData::Vertex(_), FileTypeValue::Mesh) => {
                let color = Vertex2d::CHILD_COLOR;

                let _new_vertex_2d_entity = shape_manager.vertex_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    false,
                    Some(tab_id),
                    color,
                );

                if self.on_vertex_complete(entity).is_some() {
                    panic!("mesh vertex should not have children!");
                }
            }
            (ShapeData::Edge(start, end), _) => {

                let start_2d = *shape_manager.vertex_entity_3d_to_2d(&start).unwrap();
                let end_2d = *shape_manager.vertex_entity_3d_to_2d(&end).unwrap();

                commands.entity(entity).insert(Edge3dLocal::new(start, end));

                let _new_edge_2d_entity = shape_manager.edge_3d_postprocess(
                    commands,
                    meshes,
                    materials,
                    camera_manager,
                    entity,
                    start_2d,
                    end_2d,
                    Some(tab_id),
                    Vertex2d::CHILD_COLOR,
                    file_type == FileTypeValue::Skel,
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
