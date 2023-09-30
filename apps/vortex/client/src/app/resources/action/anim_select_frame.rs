use bevy_ecs::{
    prelude::World,
    system::{Res, Commands, SystemState, ResMut},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{tab_manager::TabManager, action::AnimAction, animation_manager::AnimationManager};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::SelectFrame(next_frame_index, last_frame_index) = action else {
        panic!("Expected SelectFrame");
    };

    info!("SelectFrame({:?} -> {:?})", last_frame_index, next_frame_index);

    let mut system_state: SystemState<(Commands, Client, Res<TabManager>, ResMut<AnimationManager>)> = SystemState::new(world);
    let (mut commands, mut client, tab_manager, mut animation_manager) = system_state.get_mut(world);

    let Some(file_entity) = tab_manager.current_tab_entity() else {
        return vec![];
    };
    let file_entity = *file_entity;

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

    return vec![AnimAction::SelectFrame(last_frame_index, next_frame_index)];
}
