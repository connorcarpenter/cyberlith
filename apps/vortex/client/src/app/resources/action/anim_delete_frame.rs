use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::info;

use vortex_proto::components::AnimRotation;

use crate::app::resources::{action::AnimAction, animation_manager::AnimationManager};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::DeleteFrame(file_entity, frame_index, last_frame_index_opt) = action else {
        panic!("Expected DeleteFrame");
    };

    info!(
        "DeleteFrame({:?}, {:?}, {:?})",
        file_entity, frame_index, last_frame_index_opt
    );

    let mut system_state: SystemState<(Commands, ResMut<AnimationManager>, Query<&AnimRotation>)> =
        SystemState::new(world);
    let (mut commands, mut animation_manager, rot_q) = system_state.get_mut(world);

    let frame_entity = animation_manager
        .get_frame_entity(&file_entity, frame_index)
        .unwrap();

    let Some(rotation_entities) = animation_manager.get_frame_rotations(&file_entity, &frame_entity) else {
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

    commands.entity(frame_entity).despawn();

    animation_manager.deregister_frame(&file_entity, &frame_entity);

    system_state.apply(world);

    return vec![AnimAction::InsertFrame(
        file_entity,
        frame_index,
        Some(rotations),
    )];
}
