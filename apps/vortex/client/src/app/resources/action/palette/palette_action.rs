use bevy_ecs::prelude::{Entity, World};

use render_egui::egui::Color32;

use crate::app::resources::{
    action::{palette::{select_color, delete_color, insert_color, move_color}, Action},
    palette_manager::PaletteManager,
};

#[derive(Clone)]
pub enum PaletteAction {
    // file entity, next color index, last color index
    SelectColor(Entity, usize, usize),
    // file entity, color index
    InsertColor(Entity, usize, Option<Color32>),
    // file entity, color index
    DeleteColor(Entity, usize),
    // file entity, color index, last color index
    MoveColor(Entity, usize, usize),
}

pub enum PaletteActionType {
    SelectColor,
    InsertColor,
    DeleteColor,
    MoveColor,
}

impl PaletteAction {
    pub fn get_type(&self) -> PaletteActionType {
        match self {
            Self::SelectColor(_, _, _) => PaletteActionType::SelectColor,
            Self::InsertColor(_, _, _) => PaletteActionType::InsertColor,
            Self::DeleteColor(_, _) => PaletteActionType::DeleteColor,
            Self::MoveColor(_, _, _) => PaletteActionType::MoveColor,
        }
    }

    pub fn execute(self, world: &mut World, palette_manager: &mut PaletteManager) -> Vec<Self> {
        let action_type = self.get_type();

        match action_type {
            PaletteActionType::SelectColor => select_color::execute(world, palette_manager, self),
            PaletteActionType::InsertColor => insert_color::execute(world, palette_manager, self),
            PaletteActionType::DeleteColor => delete_color::execute(world, palette_manager, self),
            PaletteActionType::MoveColor => move_color::execute(world, palette_manager, self),
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
