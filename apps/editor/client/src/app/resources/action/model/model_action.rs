use bevy_ecs::prelude::{Entity, World};

use render_api::components::Transform;

use editor_proto::components::FileExtension;

use crate::app::resources::{
    action::{
        model::{create_transform, delete_transform, move_transform, select_shape},
        Action,
    },
    input::InputManager,
    shape_data::CanvasShape,
};

#[derive(Clone)]
pub enum ModelAction {
    // The 2D shape entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    // Option<edge_2d_entity>, dependency file ext, dependency file entity
    CreateTransform(Option<Entity>, FileExtension, Entity),
    // net_transform_entity
    DeleteTransform(Entity),
    // Move Transform (Transform Entity, Old Transform, New Transform, ?)
    MoveTransform(Entity, Transform, Transform, bool),
}

pub enum ModelActionType {
    SelectShape,
    CreateTransform,
    DeleteTransform,
    MoveTransform,
}

impl ModelAction {
    pub fn get_type(&self) -> ModelActionType {
        match self {
            Self::SelectShape(_) => ModelActionType::SelectShape,
            Self::CreateTransform(_, _, _) => ModelActionType::CreateTransform,
            Self::DeleteTransform(_) => ModelActionType::DeleteTransform,
            Self::MoveTransform(_, _, _, _) => ModelActionType::MoveTransform,
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
            ModelActionType::SelectShape => select_shape::execute(world, input_manager, self),
            ModelActionType::CreateTransform => {
                create_transform::execute(world, input_manager, &current_file_entity, self)
            }
            ModelActionType::DeleteTransform => {
                delete_transform::execute(world, &current_file_entity, self)
            }
            ModelActionType::MoveTransform => move_transform::execute(world, self),
        }
    }
}

impl Action for ModelAction {
    fn entity_update_auth_status_impl(
        _buffered_check: &mut bool,
        action_opt: Option<&Self>,
        _entity: &Entity,
    ) {
        match action_opt {
            _ => {}
        }
    }

    fn enable_top_impl(_world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            _ => {
                *enabled = true;
            }
        }
    }
}
