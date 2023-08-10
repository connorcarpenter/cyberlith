use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::Resource,
};
use bevy_log::info;

use vortex_proto::components::FileTypeValue;

use crate::resources::VertexManager;

pub enum VertexWaitlistInsert {
    ///////////Vertex
    VertexRoot(Entity),
    /////parent, edge, child
    Edge(Entity, Entity, Entity),
    /////////Vertex
    FileType(Entity, FileTypeValue),
}

enum VertexData {
    Skel(Option<(Entity, Entity)>),
    Mesh,
}

#[derive(Clone)]
pub struct VertexWaitlistEntry {
    edge_and_parent_opt: Option<Option<(Entity, Entity)>>,
    file_type: Option<FileTypeValue>,
}

impl VertexWaitlistEntry {
    fn new() -> Self {
        Self {
            edge_and_parent_opt: None,
            file_type: None,
        }
    }

    fn is_ready(&self) -> bool {
        match self.file_type {
            Some(FileTypeValue::Skel) => {
                return self.edge_and_parent_opt.is_some();
            }
            Some(FileTypeValue::Mesh) => {
                return true;
            }
            None => {
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

    fn set_file_type(&mut self, file_type: FileTypeValue) {
        self.file_type = Some(file_type);
    }

    fn decompose(self) -> VertexData {
        match self.file_type {
            Some(FileTypeValue::Skel) => {
                return VertexData::Skel(self.edge_and_parent_opt.unwrap());
            }
            Some(FileTypeValue::Mesh) => {
                return VertexData::Mesh;
            }
            None => {
                panic!("shouldn't be able to happen!");
            }
        }
    }
}

#[derive(Resource)]
pub struct VertexWaitlist {
    // incomplete entity -> entry
    incomplete_entries: HashMap<Entity, VertexWaitlistEntry>,
    // waiting entity -> (entity dependencies, entry)
    dependent_map: HashMap<Entity, (HashSet<Entity>, VertexWaitlistEntry)>,
    // entity dependency -> entities waiting on it
    dependency_map: HashMap<Entity, HashSet<Entity>>,
}

impl Default for VertexWaitlist {
    fn default() -> Self {
        Self {
            incomplete_entries: HashMap::new(),
            dependent_map: HashMap::new(),
            dependency_map: HashMap::new(),
        }
    }
}

impl VertexWaitlist {
    pub fn process_inserts(
        &mut self,
        vertex_manager: &mut VertexManager,
        inserts: Vec<VertexWaitlistInsert>,
    ) {
        for insert in inserts {
            self.process_insert(vertex_manager, insert);
        }
    }

    pub fn process_insert(
        &mut self,
        vertex_manager: &mut VertexManager,
        insert: VertexWaitlistInsert,
    ) {
        let mut possibly_ready_entities = Vec::new();

        match insert {
            VertexWaitlistInsert::Edge(parent_entity, edge_entity, vertex_entity) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, VertexWaitlistEntry::new());
                }
                let vertex_entry = self.get_mut(&vertex_entity).unwrap();
                info!("Setting parent of {:?} to {:?}", vertex_entity, parent_entity);
                vertex_entry.set_edge_and_parent(Some((edge_entity, parent_entity)));
                possibly_ready_entities.push(vertex_entity);
                info!(
                    "Entities to check for readiness... `{:?}`",
                    possibly_ready_entities
                );
            }
            VertexWaitlistInsert::VertexRoot(vertex_entity) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, VertexWaitlistEntry::new());
                }
                self.get_mut(&vertex_entity).unwrap().set_edge_and_parent(None);
                possibly_ready_entities.push(vertex_entity);
            }
            VertexWaitlistInsert::FileType(vertex_entity, file_type) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, VertexWaitlistEntry::new());
                }
                self.get_mut(&vertex_entity).unwrap().set_file_type(file_type);
                possibly_ready_entities.push(vertex_entity);
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

                if entry.has_edge_and_parent() {
                    let (_, parent_entity) = entry.get_edge_and_parent().unwrap();
                    if !vertex_manager.has_vertex(&parent_entity) {
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
                self.process_complete(
                    vertex_manager,
                    entity,
                    entry,
                );
            } else {
                info!("entity `{:?}` is not ready yet...", possibly_ready_entity);
            }
        }
    }

    fn process_complete(
        &mut self,
        vertex_manager: &mut VertexManager,
        entity: Entity,
        entry: VertexWaitlistEntry,
    ) {
        // info!("processing complete vertex {:?}", entity);

        let data = entry.decompose();

        match data {
            VertexData::Skel(edge_and_parent_opt) => {
                vertex_manager.on_create_skel_vertex(entity, edge_and_parent_opt);

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
                            vertex_manager,
                            child_entity,
                            child_entry,
                        );
                    }
                }
            }
            VertexData::Mesh => {
                vertex_manager.on_create_mesh_vertex(entity);
            }
        }
    }

    fn contains_key(&self, entity: &Entity) -> bool {
        self.incomplete_entries.contains_key(entity)
    }

    fn insert_incomplete(&mut self, entity: Entity, entry: VertexWaitlistEntry) {
        self.incomplete_entries.insert(entity, entry);
    }

    fn get_mut(&mut self, entity: &Entity) -> Option<&mut VertexWaitlistEntry> {
        self.incomplete_entries.get_mut(entity)
    }

    fn remove(&mut self, entity: &Entity) -> Option<VertexWaitlistEntry> {
        self.incomplete_entries.remove(entity)
    }

    fn insert_waiting_dependency(
        &mut self,
        dependency_entity: Entity,
        dependent_entity: Entity,
        dependent_entry: VertexWaitlistEntry,
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

    fn on_vertex_complete(&mut self, entity: Entity) -> Option<Vec<(Entity, VertexWaitlistEntry)>> {
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
