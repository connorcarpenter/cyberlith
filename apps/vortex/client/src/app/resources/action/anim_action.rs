use bevy_ecs::prelude::{Entity, World};

use math::Quat;

use crate::app::resources::{
    action::{anim_rotate_vertex, anim_select_vertex, Action, ActionStack},
    vertex_manager::VertexManager,
};

#[derive(Clone)]
pub enum AnimAction {
    // The 2D vertex entity to deselect (or None for deselect)
    SelectVertex(Option<Entity>),
    //
    RotateVertex(Entity, Option<Quat>, Option<Quat>),
}

pub enum AnimActionType {
    SelectVertex,
    RotateVertex,
}

impl AnimAction {
    pub fn get_type(&self) -> AnimActionType {
        match self {
            Self::SelectVertex(_) => AnimActionType::SelectVertex,
            Self::RotateVertex(_, _, _) => AnimActionType::RotateVertex,
        }
    }
}

impl Action for AnimAction {
    fn execute(
        self,
        world: &mut World,
        _: Option<&Entity>,
        _: &mut ActionStack<Self>,
    ) -> Vec<Self> {
        match self {
            Self::SelectVertex(vertex_2d_entity_opt) => {
                anim_select_vertex::execute(world, vertex_2d_entity_opt)
            }
            Self::RotateVertex(vertex_2d_entity, old_angle_opt, new_angle) => {
                anim_rotate_vertex::execute(world, vertex_2d_entity, old_angle_opt, new_angle)
            }
        }
    }

    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::SelectVertex(vertex_2d_entity_opt)) => {
                if let Some(vertex_2d_entity) = vertex_2d_entity_opt {
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
            Some(Self::SelectVertex(vertex_2d_entity_opt)) => {
                let mut entities = Vec::new();

                if let Some(vertex_2d_entity) = vertex_2d_entity_opt {
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
