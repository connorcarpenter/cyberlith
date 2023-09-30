use bevy_ecs::{
    prelude::World,
    system::{Commands, SystemState, ResMut},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{action::AnimAction, animation_manager::AnimationManager};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::SelectFrame(file_entity, next_frame_index, last_frame_index) = action else {
        panic!("Expected SelectFrame");
    };

    info!("SelectFrame(file `{:?}`, {:?} -> {:?})", file_entity, last_frame_index, next_frame_index);

    let mut system_state: SystemState<(Commands, Client, ResMut<AnimationManager>)> = SystemState::new(world);
    let (mut commands, mut client, mut animation_manager) = system_state.get_mut(world);

    // release the last frame entity
    let Some(last_frame_entity) = animation_manager.get_frame_entity(&file_entity, last_frame_index) else {
        return vec![];
    };
    commands.entity(last_frame_entity).release_authority(&mut client);

    animation_manager.set_current_frame_index(next_frame_index);

    // request auth over next frame entity
    let Some(next_frame_entity) = animation_manager.get_frame_entity(&file_entity, next_frame_index) else {
        return vec![];
    };
    commands.entity(next_frame_entity).request_authority(&mut client);

    system_state.apply(world);

    return vec![AnimAction::SelectFrame(file_entity, last_frame_index, next_frame_index)];
}
