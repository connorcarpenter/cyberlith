use std::collections::HashMap;

use bevy_ecs::{change_detection::{ResMut}, event::EventReader, prelude::Query};

use game_engine::{
    logging::info,
    time::Instant,
    world::{
        components::{PhysicsController, NetworkedMoveBuffer}, WorldInsertComponentEvent, WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
    naia::sequence_greater_than,
};

use crate::{systems::world_events::{PredictionEvents}, resources::{RollbackManager}, components::{RenderPosition, ConfirmedTileMovement}};

pub fn insert_net_move_buffer_events(
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NetworkedMoveBuffer>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let entity = event.entity;

        info!(
            "received Inserted Component: `NetworkedMoveBuffer` from World Server! (entity: {:?})",
            entity
        );

        prediction_events.read_insert_net_move_buffer_event(&now, &entity);
    }
}

pub fn update_net_move_buffer_events(
    mut rollback_manager: ResMut<RollbackManager>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedMoveBuffer>>,

    mut updated_q: Query<(&NetworkedMoveBuffer, &mut ConfirmedTileMovement, &mut PhysicsController, &mut RenderPosition)>,
) {
    let mut events = HashMap::new();
    for event in event_reader.read() {
        let server_tick = event.tick;
        let updated_entity = event.entity;

        if !events.contains_key(&updated_entity) {
            events.insert(updated_entity, server_tick);
        } else {
            let existing_tick = events.get(&updated_entity).unwrap();
            if sequence_greater_than(server_tick, *existing_tick) {
                events.insert(updated_entity, server_tick);
            }
        }
    }

    if events.is_empty() {
        return;
    }

    let mut rollback_events = HashMap::new();
    for (updated_entity, update_tick) in &events {
        let Ok(
            (
               net_move_buffer,
               mut tile_movement,
               mut physics,
               mut render_position
            )
        ) = updated_q.get_mut(*updated_entity) else {
            panic!(
                "failed to get tile movement q for entity: {:?}",
                updated_entity
            );
        };
        let should_rollback = tile_movement.recv_updated_net_move_buffer(
            *update_tick,
            &net_move_buffer,
            &mut physics,
            &mut render_position,
        );
        if should_rollback {
            rollback_events.insert(*updated_entity, *update_tick);
        }
    }

    rollback_manager.add_events(rollback_events);
}

pub fn remove_net_move_buffer_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NetworkedMoveBuffer>>,
) {
    for _event in event_reader.read() {
        info!("removed NetworkedMoveBuffer component from entity");
    }
}
