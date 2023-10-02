use bevy_ecs::prelude::{Entity, World};

use math::Quat;

use crate::app::resources::{
    action::{
        anim_delete_frame, anim_insert_frame, anim_rotate_vertex, anim_select_frame,
        anim_select_vertex, Action, ActionStack,
    },
    input_manager::InputManager,
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
    // file entity, frame index, last frame index
    DeleteFrame(Entity, usize, Option<usize>),
}

pub enum AnimActionType {
    SelectShape,
    RotateVertex,
    SelectFrame,
    InsertFrame,
    DeleteFrame,
}

impl AnimAction {
    pub fn get_type(&self) -> AnimActionType {
        match self {
            Self::SelectShape(_) => AnimActionType::SelectShape,
            Self::RotateVertex(_, _, _) => AnimActionType::RotateVertex,
            Self::SelectFrame(_, _, _) => AnimActionType::SelectFrame,
            Self::InsertFrame(_, _, _) => AnimActionType::InsertFrame,
            Self::DeleteFrame(_, _, _) => AnimActionType::DeleteFrame,
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
                anim_select_vertex::execute(world, input_manager, tab_file_entity, self)
            }
            AnimActionType::RotateVertex => {
                anim_rotate_vertex::execute(world, tab_file_entity, self)
            }
            AnimActionType::SelectFrame => anim_select_frame::execute(world, self),
            AnimActionType::InsertFrame => anim_insert_frame::execute(world, self),
            AnimActionType::DeleteFrame => anim_delete_frame::execute(world, self),
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
