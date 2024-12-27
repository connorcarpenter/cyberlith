use bevy_ecs::{change_detection::ResMut, event::EventReader, prelude::Query};

use game_engine::{
    logging::info,
    time::Instant,
};
use game_app_network::world::{
    components::LookDirection, WorldInsertComponentEvent, WorldRemoveComponentEvent,
    WorldUpdateComponentEvent,
};

use crate::{components::AnimationState, systems::world_events::PredictionEvents};

pub fn insert_look_direction_events(
    mut prediction_events: ResMut<PredictionEvents>,

    lookdir_q: Query<&LookDirection>,
    mut animation_state_q: Query<&mut AnimationState>,

    mut event_reader: EventReader<WorldInsertComponentEvent<LookDirection>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let entity = event.entity;

        info!(
            "received Inserted Component: `LookDirection` from World Server! (entity: {:?})",
            entity
        );

        prediction_events.read_insert_lookdir_event(&now, &entity);

        let look_direction = lookdir_q.get(entity).unwrap();
        if let Ok(mut animation_state) = animation_state_q.get_mut(entity) {
            animation_state.recv_lookdir_update(&look_direction.get());
        }
        // not sure what to do here..
    }
}

pub fn update_look_direction_events(
    look_direction_q: Query<&LookDirection>,
    mut animation_state_q: Query<&mut AnimationState>,

    mut event_reader: EventReader<WorldUpdateComponentEvent<LookDirection>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;

        info!(
            "received Updated Component: `LookDirection` from World Server! (entity: {:?})",
            entity
        );

        let look_direction = look_direction_q.get(entity).unwrap();
        if let Ok(mut animation_state) = animation_state_q.get_mut(entity) {
            animation_state.recv_lookdir_update(&look_direction.get());
        }

        // not sure what to do here..
    }
}

pub fn remove_look_direction_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<LookDirection>>,
) {
    for _event in event_reader.read() {
        info!("removed LookDirection component from entity");
    }
}
