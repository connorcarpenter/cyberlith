use std::collections::{HashMap, HashSet};

use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Commands, Resource};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, Server};

struct VertexValue {
    parent: Option<Entity>,
    children: Option<HashSet<Entity>>,
}

impl VertexValue {
    pub fn new(parent: Option<Entity>) -> Self {
        Self {
            parent,
            children: None,
        }
    }

    pub fn add_child(&mut self, entity: Entity) {
        self.children
            .get_or_insert_with(|| HashSet::new())
            .insert(entity);
    }

    pub fn remove_child(&mut self, entity: &Entity) {
        if let Some(children) = self.children.as_mut() {
            children.remove(&entity);
        }
    }
}

#[derive(Resource)]
pub struct VertexManager {
    vertices: HashMap<Entity, VertexValue>,
    waiting_for_parent: HashMap<Entity, Vec<(Entity, Option<Entity>)>>,
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

    pub fn on_create_vertex(&mut self, entity: &Entity, parent_opt: Option<Entity>) {
        // info!("on_create_vertex: {:?} {:?}", entity, parent_opt);

        let success: bool;

        if let Some(parent_entity) = parent_opt {
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

                list.push((*entity, parent_opt));
                info!(
                    "waiting on parent .. entity: {:?}, parent is {:?}",
                    entity, parent_entity
                )
            }
        } else {
            // success!
            success = true;
        }

        if success {
            self.insert_vertex(*entity, parent_opt);
        }
    }

    fn insert_vertex(&mut self, entity: Entity, parent_opt: Option<Entity>) {
        info!("inserting entity: {:?}, parent is {:?}", entity, parent_opt);
        self.vertices.insert(entity, VertexValue::new(parent_opt));

        if let Some(parent_entity) = parent_opt {
            let Some(parent_value) = self.vertices.get_mut(&parent_entity) else {
                panic!("shouldn't be able to happen!");
            };
            parent_value.add_child(entity);
        }

        if let Some(list) = self.waiting_for_parent.remove(&entity) {
            for (child_entity, child_value) in list {
                info!(
                    "child {:?} was waiting on parent {:?}!",
                    child_entity, entity
                );
                self.insert_vertex(child_entity, child_value);
            }
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

    fn remove_entity(entities: &mut HashMap<Entity, VertexValue>, entity: &Entity) -> Vec<Entity> {
        let mut output = Vec::new();

        // remove entry
        let removed_entry =
            Self::remove_entity_and_collect_children_entities(entities, entity, &mut output);

        // remove entry from parent's children
        if let Some(parent_key) = removed_entry.parent {
            if let Some(parent) = entities.get_mut(&parent_key) {
                parent.remove_child(entity);
            }
        }

        return output;
    }

    fn remove_entity_and_collect_children_entities(
        entities: &mut HashMap<Entity, VertexValue>,
        entity: &Entity,
        output: &mut Vec<Entity>,
    ) -> VertexValue {
        let removed_entry = entities.remove(entity).unwrap();

        // handle children
        if let Some(removed_entry_children) = &removed_entry.children {
            for child_entity in removed_entry_children {
                Self::remove_entity_and_collect_children_entities(entities, &child_entity, output);
                output.push(*child_entity);
            }
        }

        removed_entry
    }
}
