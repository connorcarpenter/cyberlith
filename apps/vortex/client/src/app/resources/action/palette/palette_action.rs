use bevy_ecs::prelude::{Entity, World};

use crate::app::resources::{
    action::{Action, palette::select_color}, palette_manager::PaletteManager
};

#[derive(Clone)]
pub enum PaletteAction {
    // file entity, next color index, last color index
    SelectColor(Entity, usize, usize),
}

pub enum PaletteActionType {
    SelectColor
}

impl PaletteAction {
    pub fn get_type(&self) -> PaletteActionType {
        match self {
            Self::SelectColor(_, _, _) => PaletteActionType::SelectColor,
        }
    }

    pub fn execute(
        self,
        world: &mut World,
        palette_manager: &mut PaletteManager,
    ) -> Vec<Self> {
        let action_type = self.get_type();

        match action_type {
            PaletteActionType::SelectColor => {
                select_color::execute(world, palette_manager, self)
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
