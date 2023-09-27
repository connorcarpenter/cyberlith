use bevy_ecs::prelude::{Entity, World};

use math::Quat;

use crate::app::resources::{
    action::{anim_rotate_vertex, anim_select_vertex, Action, ActionStack},
    input_manager::InputManager,
    vertex_manager::VertexManager,
};
use crate::app::resources::shape_data::CanvasShape;

#[derive(Clone)]
pub enum AnimAction {
    // The 2D vertex entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    //
    RotateVertex(Entity, Option<(Quat, f32)>, Option<(Quat, f32)>),
}

pub enum AnimActionType {
    SelectVertex,
    RotateVertex,
}

impl AnimAction {
    pub fn get_type(&self) -> AnimActionType {
        match self {
            Self::SelectShape(_) => AnimActionType::SelectVertex,
            Self::RotateVertex(_, _, _) => AnimActionType::RotateVertex,
        }
    }

    pub fn execute(self, world: &mut World, input_manager: &mut InputManager) -> Vec<Self> {
        let action_type = self.get_type();

        match action_type {
            AnimActionType::SelectVertex => anim_select_vertex::execute(world, input_manager, self),
            AnimActionType::RotateVertex => anim_rotate_vertex::execute(world, self),
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
