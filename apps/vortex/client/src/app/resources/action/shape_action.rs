
use bevy_ecs::prelude::{Entity, World};

use math::Vec3;

use crate::app::{
    components::VertexTypeData,
    resources::{action_stack::{ActionStack, Action}, shape_manager::{CanvasShape, ShapeManager}, action::{create_edge, create_vertex, delete_edge, delete_face, delete_vertex, move_vertex, select_shape}},
};

#[derive(Clone)]
pub enum ShapeAction {
    // The 2D shape entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    // Create Vertex (Vertex-specific data, Position, older vertex 2d entity & 3d entity it was associated with)
    CreateVertex(VertexTypeData, Vec3, Option<(Entity, Entity)>),
    // Delete Vertex (2d vertex entity, optional vertex 2d entity to select after delete)
    DeleteVertex(Entity, Option<(Entity, CanvasShape)>),
    // Move Vertex (2d vertex Entity, Old Position, New Position)
    MoveVertex(Entity, Vec3, Vec3),
    // Create Edge (2d vertex start entity, 2d vertex end entity, 2d shape to select, Option<Vec<(other 2d vertex entity to make a face with, old 2d face entity it was associated with)>>, Option<(older edge 2d entity)>)
    CreateEdge(
        Entity,
        Entity,
        (Entity, CanvasShape),
        Option<Vec<(Entity, Entity, bool)>>,
        Option<Entity>,
    ),
    // Delete Edge (2d edge entity, optional vertex 2d entity to select after delete)
    DeleteEdge(Entity, Option<(Entity, CanvasShape)>),
    // Delete Face (2d face entity)
    DeleteFace(Entity),
}

impl ShapeAction {

    pub(crate) fn migrate_vertex_entities(
        &mut self,
        old_2d_vert_entity: Entity,
        new_2d_vert_entity: Entity,
        old_3d_vert_entity: Entity,
        new_3d_vert_entity: Entity,
    ) {
        match self {
            ShapeAction::SelectShape(entity_opt) => match entity_opt {
                Some((entity, CanvasShape::Vertex)) | Some((entity, CanvasShape::RootVertex)) => {
                    if *entity == old_2d_vert_entity {
                        *entity = new_2d_vert_entity;
                    }
                }
                _ => {}
            },
            ShapeAction::CreateVertex(vertex_type_data, _, entity_opt) => {
                vertex_type_data.migrate_vertex_entities(
                    old_2d_vert_entity,
                    new_2d_vert_entity,
                    old_3d_vert_entity,
                    new_3d_vert_entity,
                );

                if let Some((other_2d_entity, other_3d_entity)) = entity_opt {
                    if *other_2d_entity == old_2d_vert_entity {
                        *other_2d_entity = new_2d_vert_entity;
                    }
                    if *other_3d_entity == old_3d_vert_entity {
                        *other_3d_entity = new_3d_vert_entity;
                    }
                }
            }
            ShapeAction::DeleteVertex(entity, entity_opt) => {
                if *entity == old_2d_vert_entity {
                    *entity = new_2d_vert_entity;
                }
                if let Some((other_entity, _)) = entity_opt {
                    if *other_entity == old_2d_vert_entity {
                        *other_entity = new_2d_vert_entity;
                    }
                }
            }
            ShapeAction::MoveVertex(entity, _, _) => {
                if *entity == old_2d_vert_entity {
                    *entity = new_2d_vert_entity;
                }
            }
            ShapeAction::CreateEdge(entity_a, entity_b, shape_to_select, face_to_create_opt, _) => {
                if *entity_a == old_2d_vert_entity {
                    *entity_a = new_2d_vert_entity;
                }
                if *entity_b == old_2d_vert_entity {
                    *entity_b = new_2d_vert_entity;
                }
                if let (entity, CanvasShape::Vertex) = shape_to_select {
                    if *entity == old_2d_vert_entity {
                        *entity = new_2d_vert_entity;
                    }
                }
                if let Some(entities) = face_to_create_opt {
                    for (entity, _, _) in entities {
                        if *entity == old_2d_vert_entity {
                            *entity = new_2d_vert_entity;
                        }
                    }
                }
            }
            ShapeAction::DeleteEdge(_, Some((entity, _))) => {
                if *entity == old_2d_vert_entity {
                    *entity = new_2d_vert_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_edge_entities(
        &mut self,
        old_2d_edge_entity: Entity,
        new_2d_edge_entity: Entity,
    ) {
        match self {
            ShapeAction::SelectShape(entity_opt) => match entity_opt {
                Some((entity, CanvasShape::Edge)) => {
                    if *entity == old_2d_edge_entity {
                        *entity = new_2d_edge_entity;
                    }
                }
                _ => {}
            },
            ShapeAction::CreateVertex(vertex_type_data, _, _) => {
                vertex_type_data.migrate_edge_entities(old_2d_edge_entity, new_2d_edge_entity);
            }
            ShapeAction::CreateEdge(_, _, shape_to_select, _, Some(edge_2d_entity)) => {
                if *edge_2d_entity == old_2d_edge_entity {
                    *edge_2d_entity = new_2d_edge_entity;
                }
                if let (entity, CanvasShape::Edge) = shape_to_select {
                    if *entity == old_2d_edge_entity {
                        *entity = new_2d_edge_entity;
                    }
                }
            }
            ShapeAction::DeleteEdge(edge_2d_entity, _) => {
                if *edge_2d_entity == old_2d_edge_entity {
                    *edge_2d_entity = new_2d_edge_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_face_entities(
        &mut self,
        old_2d_face_entity: Entity,
        new_2d_face_entity: Entity,
    ) {
        match self {
            ShapeAction::SelectShape(entity_opt) => match entity_opt {
                Some((face_2d_entity, CanvasShape::Face)) => {
                    if *face_2d_entity == old_2d_face_entity {
                        *face_2d_entity = new_2d_face_entity;
                    }
                }
                _ => {}
            },
            ShapeAction::CreateVertex(vertex_type_data, _, _) => {
                vertex_type_data.migrate_face_entities(old_2d_face_entity, new_2d_face_entity);
            }
            ShapeAction::CreateEdge(_, _, _, faces_to_create_opt, _) => {
                if let Some(entities) = faces_to_create_opt {
                    for (_, face_2d_entity, _) in entities {
                        if *face_2d_entity == old_2d_face_entity {
                            *face_2d_entity = new_2d_face_entity;
                        }
                    }
                }
            }
            ShapeAction::DeleteFace(face_2d_entity) => {
                if *face_2d_entity == old_2d_face_entity {
                    *face_2d_entity = new_2d_face_entity;
                }
            }
            _ => {}
        }
    }
}

impl Action for ShapeAction {
    fn execute(self, world: &mut World, tab_file_entity_opt: Option<&Entity>, action_stack: &mut ActionStack<Self>) -> Vec<Self> {
        let Some(tab_file_entity) = tab_file_entity_opt else {
            panic!("should be a tab file entity");
        };
        match self {
            ShapeAction::SelectShape(shape_2d_entity_opt) => {
                select_shape::execute(world, shape_2d_entity_opt)
            }
            ShapeAction::CreateVertex(vertex_type_data, position, old_vertex_entities_opt) => {
                create_vertex::execute(
                    world,
                    action_stack,
                    tab_file_entity,
                    vertex_type_data,
                    position,
                    old_vertex_entities_opt,
                )
            }
            ShapeAction::DeleteVertex(vertex_2d_entity, vertex_2d_to_select_opt) => {
                delete_vertex::execute(world, vertex_2d_entity, vertex_2d_to_select_opt)
            }
            ShapeAction::MoveVertex(vertex_2d_entity, old_position, new_position) => {
                move_vertex::execute(world, vertex_2d_entity, old_position, new_position)
            }
            ShapeAction::CreateEdge(
                vertex_2d_entity_a,
                vertex_2d_entity_b,
                (shape_2d_entity_to_select, shape_2d_type_to_select),
                face_to_create_opt,
                old_edge_entities_opt,
            ) => create_edge::execute(
                world,
                action_stack,
                tab_file_entity,
                vertex_2d_entity_a,
                vertex_2d_entity_b,
                (shape_2d_entity_to_select, shape_2d_type_to_select),
                face_to_create_opt,
                old_edge_entities_opt,
            ),
            ShapeAction::DeleteEdge(edge_2d_entity, shape_2d_to_select_opt) => {
                delete_edge::execute(world, edge_2d_entity, shape_2d_to_select_opt)
            }
            ShapeAction::DeleteFace(face_2d_entity) => delete_face::execute(world, face_2d_entity),
        }
    }

    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::SelectShape(vertex_2d_entity_opt)) => {
                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    if vertex_2d_entity == entity {
                        *buffered_check = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            Some(Self::SelectShape(vertex_2d_entity_opt)) => {
                let mut entities = Vec::new();

                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = world
                        .get_resource::<ShapeManager>()
                        .unwrap()
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    entities.push(vertex_3d_entity);
                }

                *enabled = ActionStack::<ShapeAction>::should_be_enabled(world, &entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }
}