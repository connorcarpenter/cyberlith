use bevy_ecs::prelude::{Entity, World};

use vortex_proto::components::FileExtension;

use crate::app::resources::{
    shape_data::CanvasShape,
    action::{
        model::{create_model_transform, select_shape, delete_model_transform},
        Action,
    },
    input::InputManager,
};

#[derive(Clone)]
pub enum ModelAction {
    // The 2D shape entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    // edge_2d_entity, dependency file ext, dependency file entity
    CreateModelTransform(Entity, FileExtension, Entity),
    // edge_2d_entity
    DeleteModelTransform(Entity),
}

pub enum ModelActionType {
    SelectShape,
    CreateModelTransform,
    DeleteModelTransform,
    None,
}

impl ModelAction {
    pub fn get_type(&self) -> ModelActionType {
        match self {
            Self::SelectShape(_) => ModelActionType::SelectShape,
            Self::CreateModelTransform(_, _, _) => ModelActionType::CreateModelTransform,
            Self::DeleteModelTransform(_) => ModelActionType::DeleteModelTransform,
        }
    }

    pub fn execute(
        self,
        world: &mut World,
        input_manager: &mut InputManager,
        current_file_entity: Entity,
    ) -> Vec<Self> {
        let action_type = self.get_type();

        match action_type {
            ModelActionType::SelectShape => {
                select_shape::execute(world, input_manager, self)
            }
            ModelActionType::CreateModelTransform => {
                create_model_transform::execute(world, input_manager, &current_file_entity, self)
            }
            ModelActionType::DeleteModelTransform => {
                delete_model_transform::execute(world, self)
            },
            _ => Vec::new(),
        }
    }
}

impl Action for ModelAction {
    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            _ => {}
        }
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            _ => {
                *enabled = true;
            }
        }
    }
}
