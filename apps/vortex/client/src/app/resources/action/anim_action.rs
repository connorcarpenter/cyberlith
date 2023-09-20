use bevy_ecs::prelude::{Entity, World};

use crate::app::{
    resources::{
        action::{
            Action, ActionStack, select_vertex_anim,
        },
        vertex_manager::VertexManager,
    },
};

#[derive(Clone)]
pub enum AnimAction {
    // The 2D vertex entity to deselect (or None for deselect)
    SelectVertex(Option<Entity>),
}

impl Action for AnimAction {
    fn execute(
        self,
        world: &mut World,
        _: Option<&Entity>,
        _: &mut ActionStack<Self>,
    ) -> Vec<Self> {
        match self {
            Self::SelectVertex(shape_2d_entity_opt) => {
                select_vertex_anim::execute(world, shape_2d_entity_opt)
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
