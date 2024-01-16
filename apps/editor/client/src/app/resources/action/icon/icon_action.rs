use bevy_ecs::prelude::{Entity, World};

use math::Vec2;

use crate::app::{
    components::IconVertexActionData,
    resources::{
        action::{
            icon::{
                create_edge, create_vertex, delete_edge, delete_face, delete_frame, delete_vertex,
                edit_color, insert_frame, move_frame, move_vertex, select_frame, select_shape,
            },
            Action, ActionStack,
        },
        icon_manager::{IconManager, IconShapeData},
        shape_data::CanvasShape,
    },
};

#[derive(Clone)]
pub enum IconAction {
    // The shape entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    // Create Vertex (Vertex data, Position, older vertex entity it was associated with)
    CreateVertex(IconVertexActionData, Vec2, Option<Entity>),
    // Delete Vertex (vertex entity, optional vertex entity to select after delete)
    DeleteVertex(Entity, Option<(Entity, CanvasShape)>),
    // Move Vertex (vertex Entity, Old Position, New Position)
    MoveVertex(Entity, Vec2, Vec2, bool),
    // Create Edge
    CreateEdge(
        // frame entity
        Entity,
        // vertex start entity
        Entity,
        // vertex end entity
        Entity,
        // shape to select
        (Entity, CanvasShape),
        //Option<Vec<(other vertex entity to make a face with, old local face entity it was associated with, old palette color entity for face)>>>
        Option<Vec<(Entity, Entity, Option<Entity>)>>,
        //Option<older edge entity>
        Option<Entity>,
    ),
    // Delete Edge (edge entity, optional vertex entity to select after delete)
    DeleteEdge(Entity, Option<(Entity, CanvasShape)>),
    // Delete Face (face entity)
    DeleteFace(Entity),

    // framing
    // file entity, next frame index, last frame index
    SelectFrame(Entity, usize, usize),
    // file entity, frame index, copied shape data
    InsertFrame(Entity, usize, Option<Vec<IconShapeData>>),
    // file entity, frame index
    DeleteFrame(Entity, usize),
    // file entity, frame index, last frame index
    MoveFrame(Entity, usize, usize),

    // colors
    // 2D face entity, new palette color entity (or None to destroy)
    EditColor(Entity, Option<Entity>),
}

pub enum IconActionType {
    SelectShape,
    CreateVertex,
    DeleteVertex,
    MoveVertex,
    CreateEdge,
    DeleteEdge,
    DeleteFace,
    SelectFrame,
    InsertFrame,
    DeleteFrame,
    MoveFrame,
    EditColor,
}

impl IconAction {
    pub(crate) fn get_type(&self) -> IconActionType {
        match self {
            Self::SelectShape(_) => IconActionType::SelectShape,
            Self::CreateVertex(_, _, _) => IconActionType::CreateVertex,
            Self::DeleteVertex(_, _) => IconActionType::DeleteVertex,
            Self::MoveVertex(_, _, _, _) => IconActionType::MoveVertex,
            Self::CreateEdge(_, _, _, _, _, _) => IconActionType::CreateEdge,
            Self::DeleteEdge(_, _) => IconActionType::DeleteEdge,
            Self::DeleteFace(_) => IconActionType::DeleteFace,
            Self::SelectFrame(_, _, _) => IconActionType::SelectFrame,
            Self::InsertFrame(_, _, _) => IconActionType::InsertFrame,
            Self::DeleteFrame(_, _) => IconActionType::DeleteFrame,
            Self::MoveFrame(_, _, _) => IconActionType::MoveFrame,
            Self::EditColor(_, _) => IconActionType::EditColor,
        }
    }

    pub fn execute(
        self,
        world: &mut World,
        icon_manager: &mut IconManager,
        current_file_entity: Entity,
        action_stack: &mut ActionStack<Self>,
    ) -> Vec<Self> {
        let action_type = self.get_type();
        match action_type {
            IconActionType::SelectShape => {
                select_shape::execute(world, icon_manager, current_file_entity, self)
            }
            IconActionType::CreateVertex => {
                create_vertex::execute(world, icon_manager, action_stack, current_file_entity, self)
            }
            IconActionType::DeleteVertex => delete_vertex::execute(world, icon_manager, self),
            IconActionType::MoveVertex => move_vertex::execute(world, icon_manager, self),
            IconActionType::CreateEdge => {
                create_edge::execute(world, icon_manager, action_stack, current_file_entity, self)
            }
            IconActionType::DeleteEdge => delete_edge::execute(world, icon_manager, self),
            IconActionType::DeleteFace => delete_face::execute(world, icon_manager, self),
            IconActionType::SelectFrame => select_frame::execute(world, icon_manager, self),
            IconActionType::InsertFrame => insert_frame::execute(world, icon_manager, self),
            IconActionType::DeleteFrame => delete_frame::execute(world, icon_manager, self),
            IconActionType::MoveFrame => move_frame::execute(world, icon_manager, self),
            IconActionType::EditColor => edit_color::execute(world, icon_manager, self),
        }
    }

    pub(crate) fn migrate_vertex_entities(
        &mut self,
        old_vertex_entity: Entity,
        new_vertex_entity: Entity,
    ) {
        match self {
            Self::SelectShape(entity_opt) => match entity_opt {
                Some((entity, CanvasShape::Vertex)) => {
                    if *entity == old_vertex_entity {
                        *entity = new_vertex_entity;
                    }
                }
                _ => {}
            },
            Self::CreateVertex(vertex_type_data, _, entity_opt) => {
                vertex_type_data.migrate_vertex_entities(old_vertex_entity, new_vertex_entity);

                if let Some(other_entity) = entity_opt {
                    if *other_entity == old_vertex_entity {
                        *other_entity = new_vertex_entity;
                    }
                }
            }
            Self::DeleteVertex(entity, entity_opt) => {
                if *entity == old_vertex_entity {
                    *entity = new_vertex_entity;
                }
                if let Some((other_entity, _)) = entity_opt {
                    if *other_entity == old_vertex_entity {
                        *other_entity = new_vertex_entity;
                    }
                }
            }
            Self::MoveVertex(entity, _, _, _) => {
                if *entity == old_vertex_entity {
                    *entity = new_vertex_entity;
                }
            }
            Self::CreateEdge(
                _,
                vertex_entity_a,
                vertex_entity_b,
                shape_to_select,
                face_to_create_opt,
                _,
            ) => {
                if *vertex_entity_a == old_vertex_entity {
                    *vertex_entity_a = new_vertex_entity;
                }
                if *vertex_entity_b == old_vertex_entity {
                    *vertex_entity_b = new_vertex_entity;
                }
                if let (entity, CanvasShape::Vertex) = shape_to_select {
                    if *entity == old_vertex_entity {
                        *entity = new_vertex_entity;
                    }
                }
                if let Some(entities) = face_to_create_opt {
                    for (entity, _, _) in entities {
                        if *entity == old_vertex_entity {
                            *entity = new_vertex_entity;
                        }
                    }
                }
            }
            Self::DeleteEdge(_, Some((entity, _))) => {
                if *entity == old_vertex_entity {
                    *entity = new_vertex_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_edge_entities(
        &mut self,
        old_edge_entity: Entity,
        new_edge_entity: Entity,
    ) {
        match self {
            Self::SelectShape(entity_opt) => match entity_opt {
                Some((entity, CanvasShape::Edge)) => {
                    if *entity == old_edge_entity {
                        *entity = new_edge_entity;
                    }
                }
                _ => {}
            },
            Self::CreateVertex(vertex_type_data, _, _) => {
                vertex_type_data.migrate_edge_entities(old_edge_entity, new_edge_entity);
            }
            Self::CreateEdge(_, _, _, shape_to_select, _, Some(edge_entity)) => {
                if *edge_entity == old_edge_entity {
                    *edge_entity = new_edge_entity;
                }
                if let (entity, CanvasShape::Edge) = shape_to_select {
                    if *entity == old_edge_entity {
                        *entity = new_edge_entity;
                    }
                }
            }
            Self::DeleteEdge(edge_entity, _) => {
                if *edge_entity == old_edge_entity {
                    *edge_entity = new_edge_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_face_entities(
        &mut self,
        old_face_entity: Entity,
        new_face_entity: Entity,
    ) {
        match self {
            Self::SelectShape(entity_opt) => match entity_opt {
                Some((face_entity, CanvasShape::Face)) => {
                    if *face_entity == old_face_entity {
                        *face_entity = new_face_entity;
                    }
                }
                _ => {}
            },
            Self::CreateVertex(vertex_type_data, _, _) => {
                vertex_type_data.migrate_face_entities(old_face_entity, new_face_entity);
            }
            Self::CreateEdge(_, _, _, _, faces_to_create_opt, _) => {
                if let Some(entities) = faces_to_create_opt {
                    for (_, face_entity, _) in entities {
                        if *face_entity == old_face_entity {
                            *face_entity = new_face_entity;
                        }
                    }
                }
            }
            Self::DeleteFace(face_entity) => {
                if *face_entity == old_face_entity {
                    *face_entity = new_face_entity;
                }
            }
            _ => {}
        }
    }
}

impl Action for IconAction {
    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::SelectShape(vertex_entity_opt)) => {
                if let Some((vertex_entity, CanvasShape::Vertex)) = vertex_entity_opt {
                    if vertex_entity == entity {
                        *buffered_check = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            Some(Self::SelectShape(shape_entity_opt)) => {
                let mut entities = Vec::new();

                if let Some((shape_entity, _)) = shape_entity_opt {
                    entities.push(*shape_entity);
                }

                *enabled = ActionStack::<Self>::should_be_enabled(world, &entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }
}
