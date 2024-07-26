use bevy_ecs::{query::With, prelude::Query, event::EventReader, change_detection::ResMut};

use game_engine::{logging::warn, world::{WorldServerTickEvent, behavior as shared_behavior, messages::KeyCommand, components::{Position, TileMovement, NextTilePosition, PrevTilePosition}, WorldClient, WorldClientTickEvent, channels::PlayerCommandChannel}};

use crate::{resources::Global, components::{Confirmed, Predicted}};

pub fn client_tick_events(
    mut client: WorldClient,
    mut global: ResMut<Global>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<(&mut PrevTilePosition, &mut NextTilePosition, &mut TileMovement, &mut Position), With<Predicted>>,
) {
    let command_opt = global.queued_command.take();

    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted) else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        // process movement
        let (
            mut prev_tile_position,
            next_tile_position,
            mut tile_movement,
            mut position,
        ) = position_q.get_mut(predicted_entity).unwrap();

        shared_behavior::process_movement(
            &mut prev_tile_position,
            &next_tile_position,
            &mut tile_movement,
            &mut position,
        );

        // process commands
        let Some(command) = command_opt.as_ref() else {
            continue;
        };
        // Command History
        if !global.command_history.can_insert(&client_tick) {
            // History is full, should this be possible??
            warn!("Command History is full, cannot insert command for tick: {:?}", client_tick);
            continue;
        }

        // Record command
        global.command_history.insert(client_tick, command.clone());

        // Send command
        client.send_tick_buffer_message::<PlayerCommandChannel, KeyCommand>(&client_tick, &command);

        let (
            prev_tile_position,
            mut next_tile_position,
            mut tile_movement,
            _position,
        ) = position_q.get_mut(predicted_entity).unwrap();

        // Apply command
        shared_behavior::process_command(
            &command,
            &prev_tile_position,
            &mut next_tile_position,
            &mut tile_movement,
        );
    }
}

pub fn server_tick_events(
    mut tick_reader: EventReader<WorldServerTickEvent>,
    mut position_q: Query<(&mut PrevTilePosition, &mut NextTilePosition, &mut TileMovement, &mut Position), With<Confirmed>>,
) {
    for _event in tick_reader.read() {
        //let server_tick = event.tick;

        // process movement
        for (
            mut prev_tile_position,
            next_tile_position,
            mut tile_movement,
            mut position,
        ) in position_q.iter_mut() {
            shared_behavior::process_movement(
                &mut prev_tile_position,
                &next_tile_position,
                &mut tile_movement,
                &mut position,
            );
        }
    }
}