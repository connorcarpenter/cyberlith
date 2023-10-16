use bevy_ecs::prelude::{Entity, World};

use crate::app::resources::{
    action::{
        model::{},
        Action,
    },
    input_manager::InputManager,
    shape_data::CanvasShape,
};

#[derive(Clone)]
pub enum ModelAction {
    None,
}

pub enum ModelActionType {
    None,
}

impl ModelAction {
    pub fn get_type(&self) -> ModelActionType {
        match self {
            Self::None => ModelActionType::None,
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
            ModelActionType::None => {
                //select_face::execute(world, input_manager, current_file_entity, self)
                Vec::new()
            }
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
