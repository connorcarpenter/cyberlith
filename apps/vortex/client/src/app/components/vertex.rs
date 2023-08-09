use bevy_ecs::{entity::Entity, prelude::Component};

use math::Vec3;
use render_api::base::Color;

// Just a marker, to distinguish from 3d version
#[derive(Component)]
pub struct Vertex2d;

impl Vertex2d {
    pub const RADIUS: f32 = 3.0;
    pub const SUBDIVISIONS: u16 = 12;
    pub const CHILD_COLOR: Color = Color::GREEN;
    pub const ROOT_COLOR: Color = Color::LIGHT_GREEN;
}

// for stored children vertexes undo/redo ...
#[derive(Clone)]
pub struct VertexEntry {
    entity_2d: Entity,
    entity_3d: Entity,
    position: Vec3,
    children: Option<Vec<VertexEntry>>,
}

impl VertexEntry {
    pub fn new(entity_2d: Entity, entity_3d: Entity, position: Vec3) -> Self {
        Self {
            entity_2d,
            entity_3d,
            position,
            children: None,
        }
    }

    pub fn set_children(&mut self, children: Vec<VertexEntry>) {
        self.children = Some(children);
    }

    pub fn entity_2d(&self) -> Entity {
        self.entity_2d
    }

    pub fn entity_3d(&self) -> Entity {
        self.entity_3d
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn children(&self) -> Option<Vec<VertexEntry>> {
        self.children.clone()
    }
}

// for the editor compass
#[derive(Component)]
pub struct Compass;

// VertexTypeData
#[derive(Clone)]
pub struct VertexTypeData(pub Entity, pub Option<Vec<VertexEntry>>);

impl VertexTypeData {
    pub fn migrate_vertex_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
        old_3d_entity: Entity,
        new_3d_entity: Entity,
    ) {
        let VertexTypeData(parent_2d_vertex_entity, children_opt) = self;
        if *parent_2d_vertex_entity == old_2d_entity {
            *parent_2d_vertex_entity = new_2d_entity;
        }
        migrate_vertex_trees(
            children_opt,
            old_2d_entity,
            new_2d_entity,
            old_3d_entity,
            new_3d_entity,
        );
    }
}

fn migrate_vertex_trees(
    vertex_trees_opt: &mut Option<Vec<VertexEntry>>,
    old_2d_entity: Entity,
    new_2d_entity: Entity,
    old_3d_entity: Entity,
    new_3d_entity: Entity,
) {
    if let Some(vertex_trees) = vertex_trees_opt {
        for vertex_tree in vertex_trees {
            if vertex_tree.entity_2d == old_2d_entity {
                vertex_tree.entity_2d = new_2d_entity;
            }
            if vertex_tree.entity_3d == old_3d_entity {
                vertex_tree.entity_3d = new_3d_entity;
            }
            migrate_vertex_trees(
                &mut vertex_tree.children,
                old_2d_entity,
                new_2d_entity,
                old_3d_entity,
                new_3d_entity,
            );
        }
    }
}
