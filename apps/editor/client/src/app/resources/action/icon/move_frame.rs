use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, SystemState},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use editor_proto::components::IconFrame;

use crate::app::{
    plugin::Main,
    resources::{action::icon::IconAction, icon_manager::IconManager},
};

pub fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::MoveFrame(file_entity, current_frame_index, next_frame_index) = action else {
        panic!("Expected MoveFrame");
    };

    info!(
        "MoveFrame(file `{:?}`, {:?} -> {:?})",
        file_entity, current_frame_index, next_frame_index
    );

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        Query<&mut IconFrame>
    )> = SystemState::new(world);
    let (mut commands, client, mut frame_q) = system_state.get_mut(world);

    // get current frame entity
    let Some(current_frame_entity) =
        icon_manager.get_frame_entity(&file_entity, current_frame_index)
    else {
        warn!(
            "Failed to get frame entity for file `{:?}` and frame index `{:?}`!",
            file_entity, current_frame_index
        );
        return vec![];
    };

    // check authority for current frame entity, should already have it
    if let Some(auth) = commands.entity(current_frame_entity).authority(&client) {
        if !auth.is_requested() && !auth.is_granted() {
            warn!(
                "Auth for current frame entity `{:?}` is denied!",
                current_frame_entity
            );
            return vec![];
        }
        // should already have authority
    }

    // change current frame to next frame order index
    let Ok(mut current_frame) = frame_q.get_mut(current_frame_entity) else {
        panic!(
            "Failed to get IconFrame for frame entity {:?}!",
            current_frame_entity
        );
    };
    // get previous order index
    let current_frame_order = current_frame.get_order();

    // check that 'current_frame_order' is equal to 'current_frame_index'
    if (current_frame_order as usize) != current_frame_index {
        panic!(
            "Expected current_frame_order to be equal to current_frame_index, but got {:?} != {:?}",
            current_frame_order, current_frame_index
        );
    }

    // set current frame order to next frame order
    info!("setting IconFrame(entity: {:?}).order to {:?} ... (previously was {:?})", current_frame_entity, next_frame_index, current_frame_order);
    current_frame.set_order(next_frame_index as u8);

    icon_manager.set_current_frame_index(next_frame_index);
    icon_manager.framing_queue_resync_frame_order(&file_entity);

    system_state.apply(world);

    return vec![IconAction::MoveFrame(
        file_entity,
        next_frame_index,
        current_frame_index,
    )];
}
