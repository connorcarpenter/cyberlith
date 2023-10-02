use bevy_ecs::{prelude::World, system::SystemState};
use bevy_ecs::system::{Commands, ResMut};
use bevy_log::info;

use crate::app::resources::action::AnimAction;
use crate::app::resources::animation_manager::AnimationManager;

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::DeleteFrame(file_entity, frame_index, last_frame_index_opt) = action else {
        panic!("Expected DeleteFrame");
    };

    info!("DeleteFrame({:?}, {:?}, {:?})", file_entity, frame_index, last_frame_index_opt);

    let mut system_state: SystemState<(Commands, ResMut<AnimationManager>)> = SystemState::new(world);
    let (mut commands, mut animation_manager) = system_state.get_mut(world);

    let frame_entity = animation_manager.get_frame_entity(&file_entity, frame_index).unwrap();

    commands.entity(frame_entity).despawn();

    animation_manager.deregister_frame(&file_entity, &frame_entity);

    system_state.apply(world);

    return vec![
        AnimAction::InsertFrame(frame_entity, frame_index),
    ];
}
