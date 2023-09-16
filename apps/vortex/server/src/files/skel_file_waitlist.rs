use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};
use bevy_log::info;

use vortex_proto::resources::DependencyMap;

use crate::{files::ShapeType, resources::ShapeManager};

pub enum SkelWaitlistInsert {
    //// shape
    Vertex(Entity),
    //// shape
    VertexRoot(Entity),
    //// parent, edge, child
    Edge(Entity, Entity, Entity),
}

enum ShapeData {
    // (Option<Edge, Parent>)
    Vertex(Option<(Entity, Entity)>),
    Edge,
}

#[derive(Clone)]
pub struct SkelWaitlistEntry {
    shape: Option<ShapeType>,
    edge_and_parent_opt: Option<Option<(Entity, Entity)>>,
}

impl SkelWaitlistEntry {
    fn new() -> Self {
        Self {
            shape: None,
            edge_and_parent_opt: None,
        }
    }

    fn is_ready(&self) -> bool {
        match self.shape {
            Some(ShapeType::Vertex) => {
                return self.edge_and_parent_opt.is_some();
            }
            Some(ShapeType::Edge) => {
                return true;
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

    fn set_shape_type(&mut self, shape_type: ShapeType) {
        self.shape = Some(shape_type);
    }

    fn decompose(self) -> ShapeData {
        match self.shape {
            Some(ShapeType::Vertex) => {
                return ShapeData::Vertex(self.edge_and_parent_opt.unwrap());
            }
            Some(ShapeType::Edge) => {
                return ShapeData::Edge;
            }
            _ => {
                panic!("shouldn't be able to happen!");
            }
        }
    }
}

#[derive(Resource)]
pub struct SkelFileWaitlist {
    // incomplete entity -> entry
    incomplete_entries: HashMap<Entity, SkelWaitlistEntry>,
    dependency_map: DependencyMap<Entity, SkelWaitlistEntry>,
}

impl Default for SkelFileWaitlist {
    fn default() -> Self {
        Self {
            incomplete_entries: HashMap::new(),
            dependency_map: DependencyMap::new(),
        }
    }
}

impl SkelFileWaitlist {
    pub fn process_insert(&mut self, shape_manager: &mut ShapeManager, insert: SkelWaitlistInsert) {
        let mut possibly_ready_entities = Vec::new();

        match insert {
            SkelWaitlistInsert::Vertex(vertex_entity) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, SkelWaitlistEntry::new());
                }
                self.get_mut(&vertex_entity)
                    .unwrap()
                    .set_shape_type(ShapeType::Vertex);
                possibly_ready_entities.push(vertex_entity);
            }
            SkelWaitlistInsert::VertexRoot(vertex_entity) => {
                if !self.contains_key(&vertex_entity) {
                    self.insert_incomplete(vertex_entity, SkelWaitlistEntry::new());
                }
                let entry = self.get_mut(&vertex_entity).unwrap();
                entry.set_edge_and_parent(None);
                entry.set_shape_type(ShapeType::Vertex);
                possibly_ready_entities.push(vertex_entity);
            }
            SkelWaitlistInsert::Edge(parent_entity, edge_entity, vertex_entity) => {
                {
                    if !self.contains_key(&vertex_entity) {
                        self.insert_incomplete(vertex_entity, SkelWaitlistEntry::new());
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
                        self.insert_incomplete(edge_entity, SkelWaitlistEntry::new());
                    }
                    let edge_entry = self.get_mut(&edge_entity).unwrap();
                    edge_entry.set_shape_type(ShapeType::Edge);
                    possibly_ready_entities.push(edge_entity);
                }

                info!(
                    "Entities to check for readiness... `{:?}`",
                    possibly_ready_entities
                );
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

                match entry.shape.unwrap() {
                    ShapeType::Vertex => {
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
                    ShapeType::Edge => {
                        info!("`{:?}` Skel Edge complete!", entity);
                    }
                    _ => {
                        panic!("shouldn't be able to happen!");
                    }
                }

                info!(
                    "processing shape type: `{:?}`, entity: `{:?}`",
                    entry.shape.unwrap(),
                    entity
                );
                self.process_complete(shape_manager, entity, entry);
            } else {
                info!("entity `{:?}` is not ready yet...", possibly_ready_entity);
            }
        }
    }

    fn process_complete(
        &mut self,
        shape_manager: &mut ShapeManager,
        entity: Entity,
        entry: SkelWaitlistEntry,
    ) {
        // info!("processing complete vertex {:?}", entity);

        let data = entry.decompose();

        match data {
            ShapeData::Vertex(edge_and_parent_opt) => {
                shape_manager.on_create_skel_vertex(entity, edge_and_parent_opt);
            }
            ShapeData::Edge => {}
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
                self.process_complete(shape_manager, child_entity, child_entry);
            }
        }
    }

    fn contains_key(&self, entity: &Entity) -> bool {
        self.incomplete_entries.contains_key(entity)
    }

    fn insert_incomplete(&mut self, entity: Entity, entry: SkelWaitlistEntry) {
        self.incomplete_entries.insert(entity, entry);
    }

    fn get_mut(&mut self, entity: &Entity) -> Option<&mut SkelWaitlistEntry> {
        self.incomplete_entries.get_mut(entity)
    }

    fn remove(&mut self, entity: &Entity) -> Option<SkelWaitlistEntry> {
        self.incomplete_entries.remove(entity)
    }
}
