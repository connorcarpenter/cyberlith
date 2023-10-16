use bevy_ecs::prelude::{Entity, World};

use crate::app::resources::{
    action::{
        skin::{edit_bckg_color, edit_color, select_face},
        Action,
    },
    input::InputManager,
    shape_data::CanvasShape,
};

#[derive(Clone)]
pub enum SkinAction {
    // The 2D face entity to deselect (or None for deselect)
    SelectFace(Option<(Entity, CanvasShape)>),
    // 2D face entity, new palette color entity (or None to destroy)
    EditColor(Entity, Option<Entity>),
    // new palette color entity (or None to destroy)
    EditBckgColor(Entity),
}

pub enum SkinActionType {
    SelectFace,
    EditColor,
    EditBckgColor,
}

impl SkinAction {
    pub fn get_type(&self) -> SkinActionType {
        match self {
            Self::SelectFace(_) => SkinActionType::SelectFace,
            Self::EditColor(_, _) => SkinActionType::EditColor,
            Self::EditBckgColor(_) => SkinActionType::EditBckgColor,
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
            SkinActionType::SelectFace => {
                select_face::execute(world, input_manager, current_file_entity, self)
            }
            SkinActionType::EditColor => edit_color::execute(world, self),
            SkinActionType::EditBckgColor => {
                edit_bckg_color::execute(world, current_file_entity, self)
            }
        }
    }
}

impl Action for SkinAction {
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
