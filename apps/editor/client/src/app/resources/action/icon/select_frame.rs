use bevy_ecs::{
    prelude::World,
    system::{Commands, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::{
    plugin::Main,
    resources::{action::icon::IconAction, icon_manager::IconManager},
};

pub fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::SelectFrame(file_entity, next_frame_index, last_frame_index) = action else {
        panic!("Expected SelectFrame");
    };

    info!(
        "SelectFrame(file `{:?}`, {:?} -> {:?})",
        file_entity, last_frame_index, next_frame_index
    );

    let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
    let (mut commands, mut client) = system_state.get_mut(world);

    // release the last frame entity
    let Some(last_frame_entity) = icon_manager.get_frame_entity(&file_entity, last_frame_index)
    else {
        return vec![];
    };
    commands
        .entity(last_frame_entity)
        .release_authority(&mut client);

    icon_manager.set_current_frame_index(next_frame_index);

    // request auth over next frame entity
    let Some(next_frame_entity) = icon_manager.get_frame_entity(&file_entity, next_frame_index)
    else {
        return vec![];
    };
    commands
        .entity(next_frame_entity)
        .request_authority(&mut client);

    system_state.apply(world);

    return vec![IconAction::SelectFrame(
        file_entity,
        last_frame_index,
        next_frame_index,
    )];
}
