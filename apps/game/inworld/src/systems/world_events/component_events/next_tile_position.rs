use std::collections::HashMap;

use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    prelude::{Commands, Query},
};

use game_engine::{
    logging::info,
    logging::warn,
    math::{Quat, Vec3},
    naia::{sequence_greater_than, Tick},
    render::components::{RenderLayers, Transform, Visibility},
    time::Instant,
    world::{
        behavior as shared_behavior,
        components::{NextTilePosition, TileMovement},
        constants::{MOVEMENT_SPEED, TILE_SIZE},
        WorldClient, WorldInsertComponentEvent, WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
};

use crate::{
    components::{AnimationState, Confirmed},
    resources::Global,
    systems::world_events::PredictionEvents,
};

pub fn insert_next_tile_position_events(
    client: WorldClient,
    mut commands: Commands,
    position_q: Query<&NextTilePosition>,
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NextTilePosition>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let tick = client.server_tick().unwrap();
        let entity = event.entity;

        info!(
            "received Inserted Component: `NextTilePosition` from World Server! (entity: {:?})",
            entity
        );

        let next_tile_position = position_q.get(entity).unwrap();

        prediction_events.read_insert_position_event(&now, &entity);

        let layer = RenderLayers::layer(0);

        commands
            .entity(entity)
            // Insert Position stuff
            .insert(TileMovement::new_stopped(next_tile_position))
            // Insert other Rendering Stuff
            .insert(AnimationState::new())
            .insert(layer)
            .insert(Visibility::default())
            .insert(
                Transform::from_translation(Vec3::splat(0.0))
                    .with_rotation(Quat::from_rotation_z(f32::to_radians(90.0))),
            )
            // mark as Confirmed
            .insert(Confirmed);
    }
}

pub fn update_next_tile_position_events(
    mut global: ResMut<Global>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NextTilePosition>>,
    next_tile_position_q: Query<&NextTilePosition>,
    mut tile_movement_q: Query<&mut TileMovement>,
) {
    // When we receive a new Position update for the Player's Entity,
    // we must ensure the Client-side Prediction also remains in-sync
    // So we roll the Prediction back to the authoritative Server state
    // and then execute all Player Commands since that tick, using the CommandHistory helper struct

    let mut updated_entities = HashMap::new();
    let mut events = Vec::new();
    for event in event_reader.read() {
        let server_tick = event.tick;
        let updated_entity = event.entity;

        if updated_entities.contains_key(&updated_entity) {
            panic!("entity already updated: {:?}", updated_entity);
        }
        updated_entities.insert(updated_entity, server_tick);
        events.push((server_tick, updated_entity));
    }

    if events.is_empty() {
        return;
    }

    for (updated_entity, update_tick) in updated_entities {
        let Ok(next_tile_position) = next_tile_position_q.get(updated_entity)
        else {
            panic!(
                "failed to get updated components for entity: {:?}",
                updated_entity
            );
        };
        let Ok(mut tile_movement) = tile_movement_q.get_mut(updated_entity)
        else {
            panic!(
                "failed to get tile movement q for entity: {:?}",
                updated_entity
            );
        };
        tile_movement.recv_updated_next_tile_position(&next_tile_position, update_tick);
    }

    let Some(owned_entity) = &global.owned_entity else {
        return;
    };
    let mut latest_tick: Option<Tick> = None;
    let server_entity = owned_entity.confirmed;
    let client_entity = owned_entity.predicted;

    for (server_tick, updated_entity) in events {
        // If entity is owned
        if updated_entity == server_entity {
            if let Some(last_tick) = &mut latest_tick {
                if sequence_greater_than(server_tick, *last_tick) {
                    *last_tick = server_tick;
                }
            } else {
                latest_tick = Some(server_tick);
            }
        }
    }

    let Some(server_tick) = latest_tick else {
        return;
    };

    let Ok([server_tile_movement, mut client_tile_movement])
        = tile_movement_q.get_many_mut([server_entity, client_entity]) else
    {
        panic!(
            "failed to get components for entities: {:?}, {:?}",
            server_entity, client_entity
        );
    };

    // Set to authoritative state
    client_tile_movement.recv_rollback(&server_tile_movement);

    // Replay all stored commands

    // TODO: why is it necessary to subtract 1 Tick here?
    // it's not like this in the Macroquad demo
    let modified_server_tick = server_tick.wrapping_sub(1);

    let replay_commands = global.command_history.replays(&modified_server_tick);

    let mut current_tick = server_tick;
    for (command_tick, command) in replay_commands {
        while sequence_greater_than(command_tick, current_tick) {
            current_tick = current_tick.wrapping_add(1);

            // PREDICTION ROLLBACK
            shared_behavior::process_movement(&mut client_tile_movement);
        }
        shared_behavior::process_command(
            &mut client_tile_movement,
            &command,
        );
    }
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NextTilePosition>>,
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}
