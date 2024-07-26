use std::collections::{HashMap};

use bevy_ecs::{prelude::{Commands, Query}, event::EventReader, change_detection::ResMut};

use game_engine::{logging::warn, time::Instant, world::{WorldClient, constants::{MOVEMENT_SPEED, TILE_SIZE}, WorldInsertComponentEvent, components::{NextTilePosition, Position, PrevTilePosition, TileMovement}, WorldRemoveComponentEvent, WorldUpdateComponentEvent, behavior as shared_behavior},
                  render::components::{RenderLayers, Transform, Visibility}, naia::{sequence_greater_than, Replicate, Tick}, math::{Quat, Vec3}, logging::info};

use crate::{systems::world_events::PredictionEvents, resources::Global, components::{BufferedNextTilePosition, Confirmed, AnimationState, Interp}};

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

        let position = Position::new(false, tick, *next_tile_position.x as f32 * TILE_SIZE, *next_tile_position.y as f32 * TILE_SIZE);
        let interp = Interp::new(&position);

        commands
            .entity(entity)

            // Insert Position stuff
            .insert(PrevTilePosition::new(*next_tile_position.x, *next_tile_position.y))
            .insert(BufferedNextTilePosition::new(*next_tile_position.x, *next_tile_position.y))
            .insert(TileMovement::new(false, tick, MOVEMENT_SPEED))
            .insert(position)
            .insert(interp)

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
    mut position_q: Query<(
        &mut PrevTilePosition,
        Option<&mut BufferedNextTilePosition>,
        &mut NextTilePosition,
        &mut TileMovement,
        &mut Position,
        &mut Interp,
    )>,
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
        let Ok(
            (
                mut prev_tile_position,
                mut buffered_tile_position,
                next_tile_position,
                mut tile_movement,
                mut position,
                mut interp,
            )
        ) = position_q.get_mut(updated_entity) else {
            panic!("failed to get updated components for entity: {:?}", updated_entity);
        };
        let buffered_tile_position = buffered_tile_position.as_mut().unwrap();
        let next_tile_changed = !buffered_tile_position.equals(&next_tile_position);

        if next_tile_changed {

            buffered_tile_position.incoming(&mut prev_tile_position, &next_tile_position);

            let x_axis_changed = *next_tile_position.x != prev_tile_position.x;
            let y_axis_changed = *next_tile_position.y != prev_tile_position.y;
            if !x_axis_changed && !y_axis_changed {
                panic!("is this possible?");
            }
            let distance = if x_axis_changed && y_axis_changed {
                std::f32::consts::SQRT_2
            } else {
                1.0
            };
            tile_movement.next(distance * TILE_SIZE);

            position.set(update_tick, prev_tile_position.x as f32 * TILE_SIZE, prev_tile_position.y as f32 * TILE_SIZE);

            interp.next_position(&position, Some("update_next_tile_position"));
        } else {
            warn!("NextTilePosition update received, but no change detected");
        }
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

    let Ok(
        [(
            server_prev_tile_position,
            _,
            server_next_tile_position,
            server_tile_movement,
            server_position,
            server_interp,
        ), (
            mut client_prev_tile_position,
            _,
            mut client_next_tile_position,
            mut client_tile_movement,
            mut client_position,
            mut client_interp,
        )]) = position_q.get_many_mut([server_entity, client_entity]) else {
        panic!("failed to get components for entities: {:?}, {:?}", server_entity, client_entity);
    };

    // Set to authoritative state
    client_prev_tile_position.mirror(&*server_prev_tile_position);
    client_next_tile_position.mirror(&*server_next_tile_position);
    client_tile_movement.mirror(&*server_tile_movement);
    client_position.mirror(&*server_position);
    client_interp.mirror(&*server_interp);

    // Replay all stored commands

    // TODO: why is it necessary to subtract 1 Tick here?
    // it's not like this in the Macroquad demo
    let modified_server_tick = server_tick.wrapping_sub(1);

    let replay_commands = global.command_history.replays(&modified_server_tick);

    let mut current_tick = server_tick;
    for (command_tick, command) in replay_commands {

        while sequence_greater_than(command_tick, current_tick) {
            current_tick = current_tick.wrapping_add(1);

            shared_behavior::process_movement(
                &mut client_prev_tile_position,
                *client_next_tile_position.x,
                *client_next_tile_position.y,
                &mut client_tile_movement,
                &mut client_position,
                current_tick,
            );
            client_interp.next_position(&client_position, Some("replay_commands"));
        }
        shared_behavior::process_command(
            &command,
            &client_prev_tile_position,
            &mut client_next_tile_position,
            &mut client_tile_movement
        );
    }
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NextTilePosition>>
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}