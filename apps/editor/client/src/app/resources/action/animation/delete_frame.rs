use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use logging::info;

use naia_bevy_client::{Client, CommandsExt};

use editor_proto::components::AnimRotation;

use crate::app::{
    plugin::Main,
    resources::{action::animation::AnimAction, animation_manager::AnimationManager},
};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::DeleteFrame(file_entity, frame_index) = action else {
        panic!("Expected DeleteFrame");
    };

    info!("DeleteFrame({:?}, {:?})", file_entity, frame_index);

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<AnimationManager>,
        Query<&AnimRotation>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut animation_manager, rot_q) = system_state.get_mut(world);

    let frame_entity = animation_manager
        .get_frame_entity(&file_entity, frame_index)
        .unwrap();

    // check auth
    if let Some(auth) = commands.entity(frame_entity).authority(&client) {
        if !auth.is_requested() && !auth.is_granted() {
            panic!(
                "current frame entity `{:?}` does not have auth!",
                frame_entity
            );
        }
    }

    // copy rotations to store in undo/redo
    let Some(rotation_entities) =
        animation_manager.get_frame_rotations(&file_entity, &frame_entity)
    else {
        panic!("Expected frame rotations");
    };

    let mut rotations = Vec::new();
    for rotation_entity in rotation_entities {
        let Ok(rot) = rot_q.get(*rotation_entity) else {
            panic!("Expected rotation");
        };
        let name = (*rot.vertex_name).clone();
        let quat = rot.get_rotation();
        rotations.push((name, quat));
    }

    // despawn
    commands.entity(frame_entity).despawn();

    // deregister
    animation_manager.deregister_frame(&file_entity, &frame_entity);

    // select frame - 1
    if frame_index > 0 {
        let next_frame_index = frame_index - 1;
        let next_frame_entity = animation_manager
            .get_frame_entity(&file_entity, next_frame_index)
            .unwrap();
        commands
            .entity(next_frame_entity)
            .request_authority(&mut client);
        animation_manager.set_current_frame_index(next_frame_index);
    }

    system_state.apply(world);

    return vec![AnimAction::InsertFrame(
        file_entity,
        frame_index,
        Some(rotations),
    )];
}
