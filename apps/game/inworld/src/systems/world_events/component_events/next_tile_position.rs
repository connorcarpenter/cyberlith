use std::collections::HashMap;

use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    prelude::{Commands, Query},
};

use game_engine::{
    logging::{info, warn},
    math::{Quat, Vec3},
    naia::{sequence_greater_than, Tick},
    render::{components::{RenderLayers, Transform, Visibility}, base::{CpuMesh, CpuMaterial}},
    time::Instant,
    world::{
        behavior as shared_behavior,
        components::{NextTilePosition, TileMovement},
        constants::{MOVEMENT_SPEED, TILE_SIZE},
        WorldClient, WorldInsertComponentEvent, WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
    storage::Storage,
};

use crate::{
    components::{AnimationState, RenderPosition, RenderHelper, Confirmed},
    resources::Global,
    systems::world_events::PredictionEvents,
};
use crate::systems::world_events::process_tick;

pub fn insert_next_tile_position_events(
    client: WorldClient,
    mut commands: Commands,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
    position_q: Query<&NextTilePosition>,
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NextTilePosition>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let server_tick = client.server_tick().unwrap();
        let server_tick_instant = client.tick_to_instant(server_tick).expect("failed to convert tick to instant");
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
            .insert(TileMovement::new_stopped(false, false, next_tile_position))
            // Insert other Rendering Stuff
            .insert(AnimationState::new())
            .insert(RenderHelper::new(&mut meshes, &mut materials))
            .insert(RenderPosition::new(next_tile_position, server_tick, server_tick_instant))
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
    client: WorldClient,
    mut global: ResMut<Global>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NextTilePosition>>,
    next_tile_position_q: Query<&NextTilePosition>,
    mut tile_movement_q: Query<(&mut TileMovement, &mut RenderPosition)>,
) {
    // When we receive a new Position update for the Player's Entity,
    // we must ensure the Client-side Prediction also remains in-sync
    // So we roll the Prediction back to the authoritative Server state
    // and then execute all Player Commands since that tick, using the CommandHistory helper struct

    let mut updated_entities = HashMap::new();
    let mut events = Vec::new();
    for event in event_reader.read() {
        let server_tick = event.tick.wrapping_sub(1); // TODO: this shouldn't be necessary to sync!
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
        let Ok((mut tile_movement, _)) = tile_movement_q.get_mut(updated_entity)
        else {
            panic!(
                "failed to get tile movement q for entity: {:?}",
                updated_entity
            );
        };
        tile_movement.recv_updated_next_tile_position(update_tick, &next_tile_position);
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

    let Ok([(
               mut server_tile_movement,
               mut server_render_position
           ), (
                mut client_tile_movement,
        mut client_render_position
    )]
    ) = tile_movement_q.get_many_mut([server_entity, client_entity]) else
    {
        panic!(
            "failed to get components for entities: {:?}, {:?}",
            server_entity, client_entity
        );
    };

    // info!("---");
    let old_server_tick = client.server_tick().unwrap();
    info!("old server tick: {:?}", old_server_tick);
    info!("updated server tick: {:?}", server_tick);

    let client_tick = client.client_tick().unwrap();
    info!("current client tick: {:?}", client_tick);

    let mut current_tick = server_tick;

    // ROLLBACK SERVER

    {
        while sequence_greater_than(old_server_tick, current_tick) || old_server_tick == current_tick {
            info!("rollback::server: tick({:?})", current_tick);
            process_tick(true, true, current_tick, &mut server_tile_movement, &mut server_render_position);
            current_tick = current_tick.wrapping_add(1);
        }
    }

    // ROLLBACK CLIENT: Replay all stored commands

    // Set to authoritative state
    client_tile_movement.recv_rollback(&server_tile_movement);
    client_render_position.recv_rollback(&client, &server_render_position);

    // PREDICTION ROLLBACK

    // info!("0. rollback::start", current_tick);

    // TODO: why is it necessary to subtract 1 Tick here?
    // it's not like this in the Macroquad demo
    let replay_commands = global.command_history.replays(&current_tick);

    for (command_tick, command) in replay_commands {

        while sequence_greater_than(command_tick, current_tick) {

            // process command (none)

            // process movement
            info!("1. rollback::movement: tick({:?})", current_tick);
            process_tick(false, true, current_tick, &mut client_tile_movement, &mut client_render_position);

            current_tick = current_tick.wrapping_add(1);
        }

        // process command
        info!("2. rollback::command: tick({:?})", command_tick);
        shared_behavior::process_command(
            &mut client_tile_movement,
            &command,
        );

        // process movement
        info!("3. rollback::movement: tick({:?})", command_tick);
        process_tick(false, true, current_tick, &mut client_tile_movement, &mut client_render_position);

        current_tick = current_tick.wrapping_add(1);
    }

    while sequence_greater_than(client_tick, current_tick) {

        // process command (none)

        // process movement
        info!("4. rollback::movement: tick({:?})", current_tick);
        process_tick(false, true, current_tick, &mut client_tile_movement, &mut client_render_position);

        current_tick = current_tick.wrapping_add(1);
    }

    client_render_position.advance_millis(&client, true, 0);
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NextTilePosition>>,
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}
