use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, SystemState},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use editor_proto::components::PaletteColor;

use crate::app::{
    plugin::Main,
    resources::{action::palette::PaletteAction, palette_manager::PaletteManager},
};

pub fn execute(
    world: &mut World,
    palette_manager: &mut PaletteManager,
    action: PaletteAction,
) -> Vec<PaletteAction> {
    let PaletteAction::MoveColor(file_entity, current_color_index, next_color_index) = action
    else {
        panic!("Expected MoveColor");
    };

    info!(
        "MoveColor(file `{:?}`, {:?} -> {:?})",
        file_entity, current_color_index, next_color_index
    );

    let mut system_state: SystemState<(Commands, Client<Main>, Query<&mut PaletteColor>)> =
        SystemState::new(world);
    let (mut commands, mut client, mut color_q) = system_state.get_mut(world);

    let Some(current_color_entity) =
        palette_manager.get_color_entity(&file_entity, current_color_index)
    else {
        warn!(
            "Failed to get color entity for file `{:?}` and color index `{:?}`!",
            file_entity, current_color_index
        );
        return vec![];
    };
    let Some(next_color_entity) = palette_manager.get_color_entity(&file_entity, next_color_index)
    else {
        warn!(
            "Failed to get color entity for file `{:?}` and color index `{:?}`!",
            file_entity, current_color_index
        );
        return vec![];
    };

    if let Some(auth) = commands.entity(current_color_entity).authority(&client) {
        if !auth.is_requested() && !auth.is_granted() {
            warn!(
                "current color entity `{:?}` does not have auth!",
                current_color_entity
            );
            return vec![];
        }
    }
    if let Some(auth) = commands.entity(next_color_entity).authority(&client) {
        if auth.is_denied() {
            warn!(
                "Auth for next frame entity `{:?}` is denied!",
                next_color_entity
            );
            return vec![];
        }
        if auth.is_available() || auth.is_releasing() {
            commands
                .entity(next_color_entity)
                .request_authority(&mut client);
        }
    }

    let Ok(next_color) = color_q.get(next_color_entity) else {
        panic!(
            "Failed to get PaletteColor for color entity {:?}!",
            next_color_entity
        );
    };
    let next_color_order = *next_color.index;

    let Ok(mut current_color) = color_q.get_mut(current_color_entity) else {
        panic!(
            "Failed to get PaletteColor for color entity {:?}!",
            current_color_entity
        );
    };
    let current_color_order = *current_color.index;
    *current_color.index = next_color_order;

    let Ok(mut next_color) = color_q.get_mut(next_color_entity) else {
        panic!(
            "Failed to get PaletteColor for color entity {:?}!",
            next_color_entity
        );
    };
    *next_color.index = current_color_order;

    palette_manager.select_color(next_color_index);
    palette_manager.queue_resync_color_order(&file_entity);

    commands
        .entity(next_color_entity)
        .release_authority(&mut client);

    system_state.apply(world);

    return vec![PaletteAction::MoveColor(
        file_entity,
        next_color_index,
        current_color_index,
    )];
}
