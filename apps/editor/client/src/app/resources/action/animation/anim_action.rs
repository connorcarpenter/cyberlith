use bevy_ecs::prelude::{Entity, World};

use math::Quat;

use crate::app::resources::{
    action::{
        animation::{
            delete_frame, insert_frame, move_frame, rotate_vertex, select_frame, select_vertex,
        },
        Action, ActionStack,
    },
    input::InputManager,
    shape_data::CanvasShape,
    vertex_manager::VertexManager,
};

#[derive(Clone)]
pub enum AnimAction {
    // The 2D vertex entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    //
    RotateVertex(Entity, Option<Quat>, Option<Quat>),
    // file entity, next frame index, last frame index
    SelectFrame(Entity, usize, usize),
    // file entity, frame index
    InsertFrame(Entity, usize, Option<Vec<(String, Quat)>>),
    // file entity, frame index
    DeleteFrame(Entity, usize),
    // file entity, frame index, last frame index
    MoveFrame(Entity, usize, usize),
}

pub enum AnimActionType {
    SelectShape,
    RotateVertex,
    SelectFrame,
    InsertFrame,
    DeleteFrame,
    MoveFrame,
}

impl AnimAction {
    pub fn get_type(&self) -> AnimActionType {
        match self {
            Self::SelectShape(_) => AnimActionType::SelectShape,
            Self::RotateVertex(_, _, _) => AnimActionType::RotateVertex,
            Self::SelectFrame(_, _, _) => AnimActionType::SelectFrame,
            Self::InsertFrame(_, _, _) => AnimActionType::InsertFrame,
            Self::DeleteFrame(_, _) => AnimActionType::DeleteFrame,
            Self::MoveFrame(_, _, _) => AnimActionType::MoveFrame,
        }
    }

    pub fn execute(
        self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
    ) -> Vec<Self> {
        let action_type = self.get_type();

        match action_type {
            AnimActionType::SelectShape => {
                select_vertex::execute(world, input_manager, tab_file_entity, self)
            }
            AnimActionType::RotateVertex => rotate_vertex::execute(world, tab_file_entity, self),
            AnimActionType::SelectFrame => select_frame::execute(world, self),
            AnimActionType::InsertFrame => insert_frame::execute(world, self),
            AnimActionType::DeleteFrame => delete_frame::execute(world, self),
            AnimActionType::MoveFrame => move_frame::execute(world, self),
        }
    }
}

impl Action for AnimAction {
    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::SelectShape(Some((vertex_2d_entity, CanvasShape::Vertex)))) => {
                if vertex_2d_entity == entity {
                    *buffered_check = true;
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
                        .get_resource::<VertexManager>()
                        .unwrap()
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    entities.push(vertex_3d_entity);
                }

                *enabled = ActionStack::<Self>::should_be_enabled(world, &entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }
}
