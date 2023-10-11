use bevy_ecs::prelude::{Entity, World};

use crate::app::resources::{action::{Action, skin::{select_face, edit_color}}, shape_data::CanvasShape, input_manager::InputManager};

#[derive(Clone)]
pub enum SkinAction {
    // The 2D face entity to deselect (or None for deselect)
    SelectFace(Option<(Entity, CanvasShape)>),
    // 2D face entity, old palette color entity, new palette color entity
    EditColor(Entity, Option<Entity>, Option<Entity>)
}

pub enum SkinActionType {
    SelectFace,
    EditColor,
}

impl SkinAction {
    pub fn get_type(&self) -> SkinActionType {
        match self {
            Self::SelectFace(_) => SkinActionType::SelectFace,
            Self::EditColor(_, _, _) => SkinActionType::EditColor,
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
            SkinActionType::EditColor => {
                edit_color::execute(world, self)
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
