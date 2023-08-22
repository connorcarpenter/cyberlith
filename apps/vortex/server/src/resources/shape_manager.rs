use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, Server};

enum VertexData {
    Skel(SkelVertexData),
    Mesh(MeshVertexData),
}

struct SkelVertexData {
    parent: Option<Entity>,
    // children map from vertex entity to edge entity
    children: Option<HashMap<Entity, Entity>>,
}

impl SkelVertexData {
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

struct MeshVertexData {
    // edge map from vertex entity to edge entity
    edges: Option<HashMap<Entity, Entity>>,
}

impl MeshVertexData {
    fn new() -> Self {
        Self { edges: None }
    }

    fn add_edge(&mut self, vertex_entity: Entity, edge_entity: Entity) {
        self.edges
            .get_or_insert_with(|| HashMap::new())
            .insert(vertex_entity, edge_entity);
    }

    // returns edge entity
    fn remove_edge(&mut self, vertex_entity: &Entity) -> Option<Entity> {
        if let Some(edge_map) = self.edges.as_mut() {
            let result = edge_map.remove(&vertex_entity);
            if edge_map.is_empty() {
                self.edges = None;
            }
            return result;
        }
        return None;
    }
}

#[derive(Resource)]
pub struct ShapeManager {
    // vertex entity -> vertex data
    vertices: HashMap<Entity, VertexData>,
    // edge entity -> connected vertex entities
    edges: HashMap<Entity, (Entity, Entity)>,
}

impl Default for ShapeManager {
    fn default() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}

impl ShapeManager {
    pub fn has_vertex(&self, entity: &Entity) -> bool {
        self.vertices.contains_key(entity)
    }

    pub fn has_edge(&self, entity: &Entity) -> bool {
        self.edges.contains_key(entity)
    }

    pub fn get_vertex_parent(&self, entity: &Entity) -> Option<Entity> {
        if let Some(vertex_data) = self.vertices.get(entity) {
            match vertex_data {
                VertexData::Skel(skel_data) => skel_data.parent,
                VertexData::Mesh(_) => {
                    panic!("should not call this on a mesh vertex!");
                }
            }
        } else {
            None
        }
    }

    pub fn on_create_mesh_vertex(&mut self, vertex_entity: Entity) {
        // info!("on_create_mesh_vertex: {:?} {:?}", entity, parent_opt);

        info!("inserting mesh vert entity: `{:?}`", vertex_entity,);

        self.vertices
            .insert(vertex_entity, VertexData::Mesh(MeshVertexData::new()));
    }

    pub fn on_create_mesh_edge(&mut self, start: Entity, edge: Entity, end: Entity) {
        self.edges.insert(edge, (start, end));

        let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&start) else {
            panic!("on_create_mesh_edge: start entity `{:?}` not found!", start);
        };
        data.add_edge(end, edge);

        let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&end) else {
            panic!("on_create_mesh_edge: end entity `{:?}` not found!", end);
        };
        data.add_edge(start, edge);
    }

    pub fn on_create_skel_vertex(
        &mut self,
        vertex_entity: Entity,
        edge_and_parent_opt: Option<(Entity, Entity)>,
    ) {
        // info!("on_create_skel_vertex: {:?} {:?}", entity, parent_opt);

        if let Some((_, parent_entity)) = edge_and_parent_opt {
            if !self.vertices.contains_key(&parent_entity) {
                panic!("on_create_skel_vertex: parent entity `{:?}` not found! Vertex Waitlist should handle this...", parent_entity);
            }
        }

        info!(
            "inserting skel vertex entity: `{:?}`, edge_and_parent is `{:?}`",
            vertex_entity, edge_and_parent_opt
        );

        if let Some((edge_entity, parent_entity)) = edge_and_parent_opt {
            self.vertices.insert(
                vertex_entity,
                VertexData::Skel(SkelVertexData::new(Some(parent_entity))),
            );
            let Some(VertexData::Skel(parent_value)) = self.vertices.get_mut(&parent_entity) else {
                panic!("shouldn't be able to happen!");
            };
            parent_value.add_child(vertex_entity, edge_entity);
        } else {
            self.vertices
                .insert(vertex_entity, VertexData::Skel(SkelVertexData::new(None)));
        }
    }

    pub fn on_delete_vertex(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        entity: &Entity,
    ) -> Vec<Entity> {
        let entities_to_despawn = match self.vertices.get(entity) {
            Some(VertexData::Skel(_)) => self.on_delete_skel_vertex(entity),
            Some(VertexData::Mesh(_)) => self.on_delete_mesh_vertex(entity),
            None => {
                panic!("on_delete_vertex: vertex entity `{:?}` not found!", entity);
            }
        };

        info!(
            "on_delete_vertex: entity `{:?}`, entities_to_despawn: `{:?}`",
            entity, entities_to_despawn,
        );

        for child_entity in entities_to_despawn.iter() {
            commands
                .entity(*child_entity)
                .take_authority(server)
                .despawn();
        }

        entities_to_despawn
    }

    pub fn on_delete_edge(&mut self, entity: &Entity) {
        let (start, end) = self.edges.remove(entity).unwrap();

        info!(
            "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
            start, entity
        );
        let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&start) else {
            panic!("shouldn't be able to happen!");
        };
        data.remove_edge(&end);

        info!(
            "removing mapping in vertex entity `{:?}`, edge entity: `{:?}`",
            end, entity
        );
        let Some(VertexData::Mesh(data)) = self.vertices.get_mut(&end) else {
            panic!("shouldn't be able to happen!");
        };
        data.remove_edge(&start);
    }

    fn on_delete_mesh_vertex(&mut self, entity: &Entity) -> Vec<Entity> {
        let edges_to_despawn = Self::remove_mesh_vertex(&mut self.vertices, entity);

        for edge_entity in edges_to_despawn.iter() {
            self.edges.remove(edge_entity);
        }

        edges_to_despawn
    }

    // returns list of edges to despawn
    fn remove_mesh_vertex(
        entities: &mut HashMap<Entity, VertexData>,
        vertex_entity: &Entity,
    ) -> Vec<Entity> {
        let mut edges_to_despawn = Vec::new();

        // remove entry
        let VertexData::Mesh(removed_entry) = entities.remove(vertex_entity).unwrap() else {
            panic!("shouldn't be able to happen!");
        };

        if let Some(edges) = removed_entry.edges {
            for (connected_vertex_entity, edge_entity) in edges {
                edges_to_despawn.push(edge_entity);
                let Some(VertexData::Mesh(connected_vertex_data)) = entities.get_mut(&connected_vertex_entity) else {
                    panic!("shouldn't be able to happen!");
                };
                connected_vertex_data.remove_edge(vertex_entity);
            }
        }

        return edges_to_despawn;
    }

    fn on_delete_skel_vertex(&mut self, entity: &Entity) -> Vec<Entity> {
        Self::remove_skel_vertex(&mut self.vertices, entity)
    }

    fn remove_skel_vertex(
        entities: &mut HashMap<Entity, VertexData>,
        vertex_entity: &Entity,
    ) -> Vec<Entity> {
        let mut entities_to_despawn = Vec::new();

        // remove entry
        let removed_entry = Self::remove_skel_vertex_and_collect_children(
            entities,
            vertex_entity,
            &mut entities_to_despawn,
        );

        // remove entry from parent's children
        if let Some(parent_key) = removed_entry.parent {
            if let Some(VertexData::Skel(parent)) = entities.get_mut(&parent_key) {
                if let Some(edge_entity) = parent.remove_child(vertex_entity) {
                    entities_to_despawn.push(edge_entity);
                }
            }
        }

        return entities_to_despawn;
    }

    fn remove_skel_vertex_and_collect_children(
        entities: &mut HashMap<Entity, VertexData>,
        entity: &Entity,
        entities_to_despawn: &mut Vec<Entity>,
    ) -> SkelVertexData {
        let VertexData::Skel(removed_entry) = entities.remove(entity).unwrap() else {
            panic!("shouldn't be able to happen!");
        };

        // handle children
        if let Some(removed_entry_children) = &removed_entry.children {
            for (child_entity, edge_entity) in removed_entry_children {
                Self::remove_skel_vertex_and_collect_children(
                    entities,
                    &child_entity,
                    entities_to_despawn,
                );
                entities_to_despawn.push(*child_entity);
                entities_to_despawn.push(*edge_entity);
            }
        }

        removed_entry
    }
}
