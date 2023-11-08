use bevy_ecs::{
    prelude::World,
    system::{Commands, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{action::icon::IconAction, icon_manager::IconManager};

pub fn execute(world: &mut World, icon_manager: &mut IconManager, action: IconAction) -> Vec<IconAction> {
    let IconAction::DeleteFrame(file_entity, frame_index) = action else {
        panic!("Expected DeleteFrame");
    };

    info!("DeleteFrame({:?}, {:?})", file_entity, frame_index);

    let mut system_state: SystemState<(
        Commands,
        Client,
    )> = SystemState::new(world);
    let (mut commands, mut client) = system_state.get_mut(world);

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
    let mut shapes = Vec::new();

    // TODO: implement equivalent!
    // let Some(rotation_entities) = icon_manager.get_frame_rotations(&file_entity, &frame_entity) else {
    //     panic!("Expected frame rotations");
    // };
    //
    // for rotation_entity in rotation_entities {
    //     let Ok(rot) = rot_q.get(*rotation_entity) else {
    //         panic!("Expected rotation");
    //     };
    //     let name = (*rot.vertex_name).clone();
    //     let quat = rot.get_rotation();
    //     shapes.push((name, quat));
    // }

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
        Some(shapes),
    )];
}
