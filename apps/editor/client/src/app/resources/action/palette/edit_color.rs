use bevy_ecs::{
    prelude::{Query, World},
    system::SystemState,
};
use logging::info;

use editor_proto::components::PaletteColor;

use crate::app::resources::action::palette::PaletteAction;

pub(crate) fn execute(world: &mut World, action: PaletteAction) -> Vec<PaletteAction> {
    let PaletteAction::EditColor(color_entity, old_color, new_color, already_edited) = action
    else {
        panic!("Expected EditColor");
    };

    info!("EditColor({:?}, _, _, {})", color_entity, already_edited);
    let mut system_state: SystemState<Query<&mut PaletteColor>> = SystemState::new(world);
    let mut color_q = system_state.get_mut(world);

    if !already_edited {
        let Ok(mut color_component) = color_q.get_mut(color_entity) else {
            panic!(
                "Failed to get PaletteColor component for vertex entity {:?}!",
                color_entity
            );
        };
        *color_component.r = new_color.r();
        *color_component.g = new_color.g();
        *color_component.b = new_color.b();
    }

    system_state.apply(world);

    return vec![PaletteAction::EditColor(
        color_entity,
        new_color,
        old_color,
        false,
    )];
}
