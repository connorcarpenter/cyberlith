use std::collections::{HashMap, HashSet};

use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Commands, Resource};

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
}

impl Default for VertexManager {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
        }
    }
}

impl VertexManager {
    pub fn entity_is_vertex(&self, entity: &Entity) -> bool {
        self.vertices.contains_key(entity)
    }

    pub fn on_client_create_vertex(&mut self, entity: &Entity, parent: Option<Entity>) {
        self.vertices.insert(*entity, VertexValue::new(parent));

        if let Some(parent) = parent {
            if let Some(parent_value) = self.vertices.get_mut(&parent) {
                parent_value.add_child(*entity);
            }
        }
    }

    pub fn on_client_delete_vertex(&mut self, commands: &mut Commands, server: &mut Server, entity: &Entity) {

        let entities_to_delete = Self::remove_entity(&mut self.vertices, entity);

        for child_entity in entities_to_delete {
            commands
                .entity(child_entity)
                .take_authority(server)
                .despawn();
        }
    }

    fn remove_entity(
        entities: &mut HashMap<Entity, VertexValue>,
        entity: &Entity,
    ) -> Vec<Entity> {
        let mut output = Vec::new();

        // remove entry
        let removed_entry = Self::remove_entity_and_collect_children_entities(entities, entity, &mut output);

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
                Self::remove_entity_and_collect_children_entities(
                    entities,
                    &child_entity,
                    output,
                );
                output.push(*child_entity);
            }
        }

        removed_entry
    }
}