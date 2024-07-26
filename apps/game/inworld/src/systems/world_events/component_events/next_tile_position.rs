use std::collections::HashSet;

use bevy_ecs::{prelude::{Commands, Query}, event::EventReader, change_detection::ResMut};

use game_engine::{time::Instant, world::{constants::{MOVEMENT_SPEED, TILE_SIZE}, WorldInsertComponentEvent, components::{NextTilePosition, Position, PrevTilePosition, TileMovement}, WorldRemoveComponentEvent, WorldUpdateComponentEvent, behavior as shared_behavior},
                  render::components::{RenderLayers, Transform, Visibility}, naia::{sequence_greater_than, Replicate, Tick}, math::{Quat, Vec3}, logging::info};

use crate::{systems::world_events::PredictionEvents, resources::Global, components::{BufferedNextTilePosition, Confirmed, AnimationState, Interp}};

pub fn insert_next_tile_position_events(
    mut commands: Commands,
    position_q: Query<&NextTilePosition>,
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NextTilePosition>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let entity = event.entity;

        info!(
            "received Inserted Component: `NextTilePosition` from World Server! (entity: {:?})",
            entity
        );

        let position = position_q.get(entity).unwrap();

        prediction_events.read_insert_position_event(&now, &entity);

        let layer = RenderLayers::layer(0);

        commands
            .entity(entity)
            .insert(layer)
            .insert(Visibility::default())
            .insert(
                Transform::from_translation(Vec3::splat(0.0))
                    .with_rotation(Quat::from_rotation_z(f32::to_radians(90.0))),
            )
            .insert(PrevTilePosition::new(*position.x, *position.y))
            .insert(BufferedNextTilePosition::new(*position.x, *position.y))
            .insert(TileMovement::new(MOVEMENT_SPEED))
            .insert(Position::new(0.0, 0.0))
            .insert(AnimationState::new())
            // initialize interpolation
            .insert(Interp::new(*position.x, *position.y))
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
        &mut Position
    )>,
) {
    // When we receive a new Position update for the Player's Entity,
    // we must ensure the Client-side Prediction also remains in-sync
    // So we roll the Prediction back to the authoritative Server state
    // and then execute all Player Commands since that tick, using the CommandHistory helper struct

    let mut updated_entities = HashSet::new();
    let mut events = Vec::new();
    for event in event_reader.read() {
        let server_tick = event.tick;
        let updated_entity = event.entity;


        updated_entities.insert(updated_entity);
        events.push((server_tick, updated_entity));
    }

    if events.is_empty() {
        return;
    }

    for updated_entity in updated_entities {
        let Ok(
            (
                mut prev_tile_position,
                mut buffered_tile_position,
                next_tile_position,
                mut tile_movement,
                _,
            )
        ) = position_q.get_mut(updated_entity) else {
            panic!("failed to get updated components for entity: {:?}", updated_entity);
        };
        let buffered_tile_position = buffered_tile_position.as_mut().unwrap();

        let last_next_tile_position = buffered_tile_position.clone();
        buffered_tile_position.x = *next_tile_position.x;
        buffered_tile_position.y = *next_tile_position.y;

        prev_tile_position.x = last_next_tile_position.x;
        prev_tile_position.y = last_next_tile_position.y;

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
            server_position
        ), (
            mut client_prev_tile_position,
            _,
            mut client_next_tile_position,
            mut client_tile_movement,
            mut client_position,
        )]) = position_q.get_many_mut([server_entity, client_entity]) else {
        panic!("failed to get components for entities: {:?}, {:?}", server_entity, client_entity);
    };

    // Set to authoritative state
    client_prev_tile_position.mirror(&*server_prev_tile_position);
    client_next_tile_position.mirror(&*server_next_tile_position);
    client_tile_movement.mirror(&*server_tile_movement);
    client_position.mirror(&*server_position);

    // Replay all stored commands

    // TODO: why is it necessary to subtract 1 Tick here?
    // it's not like this in the Macroquad demo
    let modified_server_tick = server_tick.wrapping_sub(1);

    let replay_commands = global.command_history.replays(&modified_server_tick);
    for (_command_tick, command) in replay_commands {
        shared_behavior::process_movement(&mut client_prev_tile_position, &client_next_tile_position, &mut client_tile_movement, &mut client_position);
        shared_behavior::process_command(&command, &client_prev_tile_position, &mut client_next_tile_position, &mut client_tile_movement);
    }
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NextTilePosition>>
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}