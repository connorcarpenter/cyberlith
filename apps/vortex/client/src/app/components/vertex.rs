use bevy_ecs::{entity::Entity, prelude::Component};

use math::Vec3;
use render_api::base::Color;

use vortex_proto::components::FileTypeValue;

// Just a marker, to distinguish from 3d version
#[derive(Component)]
pub struct Vertex2d;

impl Vertex2d {
    pub const RADIUS: f32 = 3.0;
    pub const DETECT_RADIUS: f32 = Vertex2d::RADIUS + 1.0;
    pub const HOVER_RADIUS: f32 = Vertex2d::RADIUS + 1.0;

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

impl Compass {
    pub const VERTEX_RADIUS: f32 = 3.0;
    pub const EDGE_THICKNESS: f32 = 1.0;
}

// VertexTypeData
#[derive(Clone)]
pub enum VertexTypeData {
    // parent_vertex_2d_entity, children_tree_opt
    Skel(Entity, Option<Vec<VertexEntry>>),
    // Vec<connected 2d vertex, optional old edge 2d entity>, Vec<2d vertex pair with which to form a face, old face 2d entity>
    Mesh(Vec<(Entity, Option<Entity>)>, Vec<(Entity, Entity, Entity, bool)>),
}

impl VertexTypeData {
    pub fn to_file_type_value(&self) -> FileTypeValue {
        match self {
            VertexTypeData::Skel(_, _) => FileTypeValue::Skel,
            VertexTypeData::Mesh(_, _) => FileTypeValue::Mesh,
        }
    }

    pub fn migrate_vertex_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
        old_3d_entity: Entity,
        new_3d_entity: Entity,
    ) {
        match self {
            VertexTypeData::Skel(parent_2d_vertex_entity, children_opt) => {
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
            VertexTypeData::Mesh(connected_vertices, connected_face_vertices) => {
                for (connected_vertex, _) in connected_vertices {
                    if *connected_vertex == old_2d_entity {
                        *connected_vertex = new_2d_entity;
                    }
                }
                for (connected_vertex_a, connected_vertex_b, _, _) in connected_face_vertices {
                    if *connected_vertex_a == old_2d_entity {
                        *connected_vertex_a = new_2d_entity;
                    }
                    if *connected_vertex_b == old_2d_entity {
                        *connected_vertex_b = new_2d_entity;
                    }
                }
            }
        }
    }

    pub fn migrate_edge_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
    ) {
        match self {
            VertexTypeData::Skel(_, _) => {}
            VertexTypeData::Mesh(connected_vertices, _) => {
                for (_, connected_edge_opt) in connected_vertices {
                    if let Some(connected_edge) = connected_edge_opt {
                        if *connected_edge == old_2d_entity {
                            *connected_edge = new_2d_entity;
                        }
                    }
                }
            }
        }
    }

    pub fn migrate_face_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
    ) {
        match self {
            VertexTypeData::Skel(_, _) => {}
            VertexTypeData::Mesh(_, connected_faces) => {
                for (_, _, face_2d_entity, _) in connected_faces {
                    if *face_2d_entity == old_2d_entity {
                        *face_2d_entity = new_2d_entity;
                    }
                }
            }
        }
    }

    // pub fn remove_vertex_entity(&mut self, entity_2d: Entity, entity_3d: Entity) -> bool {
    //     match self {
    //         VertexTypeData::Skel(parent_2d_vertex_entity, children_opt) => {
    //             if *parent_2d_vertex_entity == entity_2d {
    //                 return true;
    //             }
    //             remove_entity_from_vertex_trees(children_opt, entity_2d, entity_3d);
    //         }
    //         VertexTypeData::Mesh(connected_vertices, connected_face_vertices) => {
    //             connected_vertices.retain(|&x| x != entity_2d);
    //             connected_face_vertices.retain(|(a, b, _)| *a != entity_2d && *b != entity_2d);
    //         }
    //     }
    //     return false;
    // }
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

// fn remove_entity_from_vertex_trees(
//     vertex_trees_opt: &mut Option<Vec<VertexEntry>>,
//     entity_2d: Entity,
//     entity_3d: Entity,
// ) {
//     if let Some(vertex_trees) = vertex_trees_opt {
//         vertex_trees.retain(|x| x.entity_2d != entity_2d && x.entity_3d != entity_3d);
//         for vertex_tree in vertex_trees {
//             remove_entity_from_vertex_trees(&mut vertex_tree.children, entity_2d, entity_3d);
//         }
//     }
// }
