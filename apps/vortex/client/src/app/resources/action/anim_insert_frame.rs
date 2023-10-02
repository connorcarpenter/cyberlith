use bevy_ecs::{
    prelude::World,
    system::{Commands, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{action::AnimAction, animation_manager::AnimationManager};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::InsertFrame(file_entity, frame_index, content_opt) = action else {
        panic!("Expected InsertFrame");
    };

    info!("InsertFrame({:?}, {:?}, {:?})", file_entity, frame_index, content_opt);

    let mut system_state: SystemState<(Commands, Client, ResMut<AnimationManager>)> =
        SystemState::new(world);
    let (mut commands, mut client, mut animation_manager) = system_state.get_mut(world);

    let last_frame_index = animation_manager.current_frame_index();
    info!("current frame index: {}", last_frame_index);

    let last_frame_entity = animation_manager
        .get_frame_entity(&file_entity, last_frame_index)
        .unwrap();
    commands
        .entity(last_frame_entity)
        .release_authority(&mut client);

    let new_frame_entity = animation_manager.framing_insert_frame(
        &mut commands,
        &mut client,
        file_entity,
        frame_index,
    );

    animation_manager.set_current_frame_index(frame_index);

    if let Some(content) = content_opt {
        for (name, quat) in content {
            animation_manager.create_networked_rotation(
                &mut commands,
                &mut client,
                new_frame_entity,
                name,
                quat,
            );
        }
    }

    // TODO: migrate undo/redo entities

    // auth already granted for this frame

    system_state.apply(world);

    return vec![AnimAction::DeleteFrame(
        file_entity,
        frame_index,
        Some(last_frame_index),
    )];
}
