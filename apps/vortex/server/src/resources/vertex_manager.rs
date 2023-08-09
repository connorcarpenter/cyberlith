use std::collections::{HashMap, HashSet};

use bevy_ecs::{entity::Entity, system::{Commands, Resource}};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, Server};

struct VertexData {
    parent: Option<Entity>,
    // children map from vertex entity to edge entity
    children: Option<HashMap<Entity, Entity>>,
}

impl VertexData {
    fn new(parent: Option<Entity>) -> Self {
        Self {
            parent,
            children: None,
        }
    }

    fn add_child(&mut self, vertex_entity: Entity, edge_entity: Entity) {
        self.children
            .get_or_insert_with(|| HashMap::new())
            .insert(vertex_entity, edge_entity);
    }

    fn remove_child(&mut self, entity: &Entity) -> Option<Entity> {
        if let Some(children) = self.children.as_mut() {
            return children.remove(&entity);
        }
        return None;
    }
}

#[derive(Resource)]
pub struct VertexManager {
    vertices: HashMap<Entity, VertexData>,
    // map from parent entity -> list of (vertex entity, Option<(edge entity, parent entity)>)
    waiting_for_parent: HashMap<Entity, Vec<(Entity, Option<(Entity, Entity)>)>>,
}

impl Default for VertexManager {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
            waiting_for_parent: HashMap::new(),
        }
    }
}

impl VertexManager {

    pub fn entity_is_vertex(&self, entity: &Entity) -> bool {
        self.vertices.contains_key(entity)
    }

    pub fn get_vertex_parent(&self, entity: &Entity) -> Option<Entity> {
        if let Some(vertex_data) = self.vertices.get(entity) {
            vertex_data.parent
        } else {
            None
        }
    }

    pub fn on_create_vertex(&mut self, vertex_entity: Entity, edge_and_parent_opt: Option<(Entity, Entity)>) {
        // info!("on_create_vertex: {:?} {:?}", entity, parent_opt);

        let success: bool;

        if let Some((edge_entity, parent_entity)) = edge_and_parent_opt {
            if self.vertices.contains_key(&parent_entity) {
                // success!
                success = true;
            } else {
                // waiting on parent
                success = false;

                if !self.waiting_for_parent.contains_key(&parent_entity) {
                    self.waiting_for_parent.insert(parent_entity, Vec::new());
                }

                let Some(list) = self.waiting_for_parent.get_mut(&parent_entity) else {
                    panic!("shouldn't be able to happen!");
                };

                list.push((vertex_entity, edge_and_parent_opt));
                info!(
                    "waiting on parent .. entity: {:?}, parent is {:?}",
                    vertex_entity, parent_entity
                )
            }
        } else {
            // success!
            success = true;
        }

        if success {
            self.insert_vertex(vertex_entity, edge_and_parent_opt);
        }
    }

    pub fn finalize_vertex_creation(&self) {
        if !self.waiting_for_parent.is_empty() {
            panic!("finalize_vertex_creation: waiting_for_parent is not empty!");
        }
    }

    pub fn on_delete_vertex(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        entity: &Entity,
    ) {
        let entities_to_delete = Self::remove_entity(&mut self.vertices, entity);
        info!(
            "on_delete_vertex: entity `{:?}`, entities_to_delete: `{:?}`",
            entity, entities_to_delete,
        );

        for child_entity in entities_to_delete {
            commands
                .entity(child_entity)
                .take_authority(server)
                .despawn();
        }
    }

    fn insert_vertex(&mut self, vertex_entity: Entity, edge_and_parent_opt: Option<(Entity, Entity)>) {
        info!("inserting entity: `{:?}`, edge_and_parent is `{:?}`", vertex_entity, edge_and_parent_opt);

        if let Some((edge_entity, parent_entity)) = edge_and_parent_opt {
            self.vertices.insert(vertex_entity, VertexData::new(Some(parent_entity)));
            let Some(parent_value) = self.vertices.get_mut(&parent_entity) else {
                panic!("shouldn't be able to happen!");
            };
            parent_value.add_child(vertex_entity, edge_entity);
        } else {
            self.vertices.insert(vertex_entity, VertexData::new(None));
        }

        if let Some(list) = self.waiting_for_parent.remove(&vertex_entity) {
            for (child_entity, child_edge_and_parent_opt) in list {
                info!(
                    "child {:?} was waiting on parent {:?}!",
                    child_entity, vertex_entity
                );
                self.insert_vertex(child_entity, child_edge_and_parent_opt);
            }
        }
    }

    fn remove_entity(entities: &mut HashMap<Entity, VertexData>, entity: &Entity) -> Vec<Entity> {
        let mut entities_to_despawn = Vec::new();

        // remove entry
        let removed_entry =
            Self::remove_entity_and_collect_children_entities(entities, entity, &mut entities_to_despawn);

        // remove entry from parent's children
        if let Some(parent_key) = removed_entry.parent {
            if let Some(parent) = entities.get_mut(&parent_key) {
                if let Some(edge_entity) = parent.remove_child(entity) {
                    entities_to_despawn.push(edge_entity);
                }
            }
        }

        return entities_to_despawn;
    }

    fn remove_entity_and_collect_children_entities(
        entities: &mut HashMap<Entity, VertexData>,
        entity: &Entity,
        entities_to_despawn: &mut Vec<Entity>,
    ) -> VertexData {
        let removed_entry = entities.remove(entity).unwrap();

        // handle children
        if let Some(removed_entry_children) = &removed_entry.children {
            for (child_entity, edge_entity) in removed_entry_children {
                Self::remove_entity_and_collect_children_entities(entities, &child_entity, entities_to_despawn);
                entities_to_despawn.push(*child_entity);
                entities_to_despawn.push(*edge_entity);
            }
        }

        removed_entry
    }
}
