use bevy_ecs::prelude::{Entity, World};

use crate::app::resources::{action::Action, input::InputManager};

#[derive(Clone)]
pub enum SceneAction {
    None,
}

pub enum SceneActionType {
    None,
}

impl SceneAction {
    pub fn get_type(&self) -> SceneActionType {
        match self {
            Self::None => SceneActionType::None,
        }
    }

    pub fn execute(
        self,
        _world: &mut World,
        _input_manager: &mut InputManager,
        _current_file_entity: Entity,
    ) -> Vec<Self> {
        let action_type = self.get_type();

        match action_type {
            SceneActionType::None => {
                //select_face::execute(world, input_manager, current_file_entity, self)
                Vec::new()
            }
        }
    }
}

impl Action for SceneAction {
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
