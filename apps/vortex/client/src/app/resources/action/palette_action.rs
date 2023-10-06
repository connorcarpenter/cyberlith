use bevy_ecs::prelude::{Entity, World};

use crate::app::resources::{
    action::Action,
    input_manager::InputManager,
};

#[derive(Clone)]
pub enum PaletteAction {
    None
}

pub enum PaletteActionType {
    None
}

impl PaletteAction {
    pub fn get_type(&self) -> PaletteActionType {
        match self {
            Self::None => PaletteActionType::None,
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
            PaletteActionType::None => {
                Vec::new()
            }
        }
    }
}

impl Action for PaletteAction {
    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::None) => {

            }
            _ => {}
        }
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            Some(PaletteAction::None) => {

            }
            _ => {

            }
        }
    }
}
