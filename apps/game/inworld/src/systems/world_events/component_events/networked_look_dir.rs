use bevy_ecs::{event::EventReader, prelude::Query};

use game_engine::{logging::info};

use game_app_network::world::{
    components::NetworkedLookDir, WorldInsertComponentEvent, WorldRemoveComponentEvent,
    WorldUpdateComponentEvent,
};

use crate::{
    components::AnimationState,
};

pub fn insert_net_look_dir_events(
    lookdir_q: Query<&NetworkedLookDir>,
    mut animation_state_q: Query<&mut AnimationState>,

    mut event_reader: EventReader<WorldInsertComponentEvent<NetworkedLookDir>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;

        info!(
            "received Inserted Component: `NetworkedLookDir` from World Server! (entity: {:?})",
            entity
        );

        let look_direction = lookdir_q.get(entity).unwrap();
        if let Ok(mut animation_state) = animation_state_q.get_mut(entity) {
            animation_state.recv_lookdir_update(&look_direction.get());
        }
        // not sure what to do here..
    }
}

pub fn update_net_look_dir_events(
    look_direction_q: Query<&NetworkedLookDir>,
    mut animation_state_q: Query<&mut AnimationState>,

    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedLookDir>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;

        info!(
            "received Updated Component: `NetworkedLookDir` from World Server! (entity: {:?})",
            entity
        );

        let look_direction = look_direction_q.get(entity).unwrap();
        if let Ok(mut animation_state) = animation_state_q.get_mut(entity) {
            animation_state.recv_lookdir_update(&look_direction.get());
        }

        // not sure what to do here..
    }
}

pub fn remove_net_look_dir_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NetworkedLookDir>>,
) {
    for _event in event_reader.read() {
        info!("removed NetworkedLookDir component from entity");
    }
}
