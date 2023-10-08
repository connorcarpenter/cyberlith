use bevy_ecs::{
    prelude::World,
    system::{Commands, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{action::palette::PaletteAction, palette_manager::PaletteManager};

pub fn execute(
    world: &mut World,
    palette_manager: &mut PaletteManager,
    action: PaletteAction,
) -> Vec<PaletteAction> {
    let PaletteAction::SelectColor(file_entity, next_color_index, last_color_index) = action else {
        panic!("Expected SelectColor");
    };

    info!(
        "SelectColor(file `{:?}`, {:?} -> {:?})",
        file_entity, last_color_index, next_color_index
    );

    let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
    let (mut commands, mut client) = system_state.get_mut(world);

    // release the last color entity
    let Some(last_color_entity) = palette_manager.get_color_entity(&file_entity, last_color_index) else {
        return vec![];
    };
    commands
        .entity(last_color_entity)
        .release_authority(&mut client);

    palette_manager.select_color(next_color_index);

    // request auth over next color entity
    let Some(next_color_entity) = palette_manager.get_color_entity(&file_entity, next_color_index) else {
        return vec![];
    };
    commands
        .entity(next_color_entity)
        .request_authority(&mut client);

    system_state.apply(world);

    return vec![PaletteAction::SelectColor(
        file_entity,
        last_color_index,
        next_color_index,
    )];
}
