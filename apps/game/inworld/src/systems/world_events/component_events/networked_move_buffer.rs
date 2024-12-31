use std::collections::HashMap;

use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::EventReader,
    prelude::Query,
};

use game_engine::{logging::info, asset::{AssetHandle, AssetManager, UnitData}};

use game_app_network::{
    naia::sequence_greater_than,
    world::{
        WorldClient, components::{NetworkedMoveBuffer, PhysicsController},
        WorldInsertComponentEvent, WorldRemoveComponentEvent, WorldUpdateComponentEvent,
    },
};

use crate::{
    components::{AnimationState, ConfirmedTileMovement, RenderPosition},
    resources::{RollbackManager, TickTracker},
};

pub fn insert_net_move_buffer_events(
    client: WorldClient,
    mut rollback_manager: ResMut<RollbackManager>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NetworkedMoveBuffer>>,
) {
    let Some(server_tick) = client.server_tick() else {
        return;
    };
    for event in event_reader.read() {

        let entity = event.entity;

        info!(
            "received Inserted Component: `NetworkedMoveBuffer` from World Server! (entity: {:?})",
            entity
        );

        rollback_manager.add_event(server_tick);
    }
}

pub fn update_net_move_buffer_events(
    asset_manager: Res<AssetManager>,
    mut rollback_manager: ResMut<RollbackManager>,
    tick_tracker: Res<TickTracker>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedMoveBuffer>>,

    mut updated_q: Query<(
        &NetworkedMoveBuffer,
        &mut ConfirmedTileMovement,
        &mut PhysicsController,
        &mut RenderPosition,
        &mut AnimationState,
        &AssetHandle<UnitData>,
    )>,
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
        let Ok((
            net_move_buffer,
            mut tile_movement,
            mut physics,
            mut render_position,
            mut animation_state,
            unit_handle,
        )) = updated_q.get_mut(*updated_entity)
        else {
            continue;
        };
        let (should_rollback, must_handle_late_update) = tile_movement.recv_updated_net_move_buffer(
            *update_tick,
            &net_move_buffer,
            &mut physics,
            &mut render_position,
        );
        if must_handle_late_update {
            tile_movement.handle_late_update(
                &asset_manager,
                &tick_tracker,
                *update_tick,
                &mut physics,
                &mut render_position,
                &mut animation_state,
                &unit_handle,
            );
        }
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

    // TODO: trigger rollback!
}
