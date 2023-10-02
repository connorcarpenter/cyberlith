use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use vortex_proto::components::AnimFrame;

use crate::app::resources::{action::AnimAction, animation_manager::AnimationManager};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::MoveFrame(file_entity, current_frame_index, next_frame_index) = action else {
        panic!("Expected MoveFrame");
    };

    info!(
        "MoveFrame(file `{:?}`, {:?} -> {:?})",
        file_entity, current_frame_index, next_frame_index
    );

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<AnimationManager>,
        Query<&mut AnimFrame>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut animation_manager, mut frame_q) =
        system_state.get_mut(world);

    let Some(current_frame_entity) = animation_manager.get_frame_entity(&file_entity, current_frame_index) else {
        warn!("Failed to get frame entity for file `{:?}` and frame index `{:?}`!",file_entity, current_frame_index);
        return vec![];
    };
    let Some(next_frame_entity) = animation_manager.get_frame_entity(&file_entity, next_frame_index) else {
        warn!("Failed to get frame entity for file `{:?}` and frame index `{:?}`!",file_entity, next_frame_index);
        return vec![];
    };

    if let Some(auth) = commands.entity(current_frame_entity).authority(&client) {
        if !auth.is_requested() && !auth.is_granted() {
            warn!(
                "current frame entity `{:?}` does not have auth!",
                current_frame_entity
            );
            return vec![];
        }
    }
    if let Some(auth) = commands.entity(next_frame_entity).authority(&client) {
        if auth.is_denied() {
            warn!(
                "Auth for next frame entity `{:?}` is denied!",
                next_frame_entity
            );
            return vec![];
        }
        if auth.is_available() || auth.is_releasing() {
            commands
                .entity(next_frame_entity)
                .request_authority(&mut client);
        }
    }

    let Ok(next_frame) = frame_q.get(next_frame_entity) else {
        panic!("Failed to get AnimFrame for frame entity {:?}!", next_frame_entity);
    };
    let next_frame_order = next_frame.get_order();

    let Ok(mut current_frame) = frame_q.get_mut(current_frame_entity) else {
        panic!("Failed to get AnimFrame for frame entity {:?}!", current_frame_entity);
    };
    let current_frame_order = current_frame.get_order();
    current_frame.set_order(next_frame_order);

    let Ok(mut next_frame) = frame_q.get_mut(next_frame_entity) else {
        panic!("Failed to get AnimFrame for frame entity {:?}!", next_frame_entity);
    };
    next_frame.set_order(current_frame_order);

    animation_manager.set_current_frame_index(next_frame_index);
    animation_manager.framing_queue_resync_frame_order(&file_entity);

    commands
        .entity(next_frame_entity)
        .release_authority(&mut client);

    system_state.apply(world);

    return vec![AnimAction::MoveFrame(
        file_entity,
        next_frame_index,
        current_frame_index,
    )];
}
