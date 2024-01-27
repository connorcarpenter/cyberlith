use bevy_ecs::{
    prelude::World,
    system::{Commands, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::{
    plugin::Main,
    resources::{action::icon::IconAction, icon_manager::IconManager, input::IconInputManager},
};

pub fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::DeleteFrame(file_entity, frame_index) = action else {
        panic!("Expected DeleteFrame");
    };

    info!("DeleteFrame({:?}, {:?})", file_entity, frame_index);

    let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
    let (mut commands, client) = system_state.get_mut(world);

    let frame_entity = icon_manager
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
    let copied_shapes = IconInputManager::pack_shape_data(world, icon_manager, &file_entity);

    let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
    let (mut commands, mut client) = system_state.get_mut(world);

    // despawn
    commands.entity(frame_entity).despawn();

    // deregister
    icon_manager.deregister_frame(&file_entity, &frame_entity);

    // select frame - 1
    if frame_index > 0 {
        let next_frame_index = frame_index - 1;
        let next_frame_entity = icon_manager
            .get_frame_entity(&file_entity, next_frame_index)
            .unwrap();
        commands
            .entity(next_frame_entity)
            .request_authority(&mut client);
        icon_manager.set_current_frame_index(next_frame_index);
    }

    system_state.apply(world);

    return vec![IconAction::InsertFrame(
        file_entity,
        frame_index,
        Some(copied_shapes),
    )];
}
