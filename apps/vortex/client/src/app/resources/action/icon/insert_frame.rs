use bevy_ecs::{
    entity::Entity,
    prelude::World,
    system::{Commands, SystemState},
};

use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{action::icon::IconAction, icon_manager::IconManager};

pub fn execute(world: &mut World, icon_manager: &mut IconManager, action: IconAction) -> Vec<IconAction> {
    let IconAction::InsertFrame(file_entity, frame_index, content_opt) = action else {
        panic!("Expected InsertFrame");
    };

    info!(
        "InsertFrame({:?}, {:?}, {:?})",
        file_entity, frame_index, content_opt
    );

    let last_frame_index: usize;
    let new_frame_entity: Entity;
    let mut entities_to_release = Vec::new();

    {
        let mut system_state: SystemState<(Commands, Client)> =
            SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        last_frame_index = icon_manager.current_frame_index();
        info!("current frame index: {}", last_frame_index);

        let last_frame_entity = icon_manager
            .get_frame_entity(&file_entity, last_frame_index)
            .unwrap();
        commands
            .entity(last_frame_entity)
            .release_authority(&mut client);

        new_frame_entity = icon_manager.framing_insert_frame(
            &mut commands,
            &mut client,
            file_entity,
            frame_index,
        );

        // TODO: implement equivalent!
        // if let Some(content) = content_opt {
        //     for (name, quat) in content {
        //         let new_rot_entity = icon_manager.create_networked_rotation(
        //             &mut commands,
        //             &mut client,
        //             new_frame_entity,
        //             name,
        //             quat,
        //         );
        //         entities_to_release.push(new_rot_entity);
        //     }
        // }

        icon_manager.set_current_frame_index(frame_index);

        // TODO: migrate undo/redo entities

        system_state.apply(world);
    }

    {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        for entity in entities_to_release {
            commands.entity(entity).release_authority(&mut client);
        }

        system_state.apply(world);
    }

    return vec![IconAction::DeleteFrame(file_entity, frame_index)];
}
