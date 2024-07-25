use bevy_ecs::{prelude::Query, event::EventReader, change_detection::ResMut};

use game_engine::{logging::warn, world::{behavior as shared_behavior, messages::KeyCommand, components::{Position, TileMovement, NextTilePosition, PrevTilePosition}, WorldClient, WorldClientTickEvent, channels::PlayerCommandChannel}};

use crate::resources::Global;

pub fn tick_events(
    mut client: WorldClient,
    mut global: ResMut<Global>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<(&mut PrevTilePosition, &mut NextTilePosition, &mut TileMovement, &mut Position)>,
) {
    let command_opt = global.queued_command.take();

    for event in tick_reader.read() {
        let client_tick = event.tick;

        // process movement
        for (
                mut prev_tile_position,
                next_tile_position,
                mut tile_movement,
                mut position,
        ) in position_q.iter_mut()
        {
            shared_behavior::process_movement(
                &mut prev_tile_position,
                &next_tile_position,
                &mut tile_movement,
                &mut position,
            );
        }

        if let Some(predicted_entity) = global
            .owned_entity
            .as_ref()
            .map(|owned_entity| owned_entity.predicted) {

            // process commands
            if let Some(command) = command_opt.as_ref() {
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

                if let Ok(
                    (
                        prev_tile_position,
                        mut next_tile_position,
                        mut tile_movement,
                        _position,
                    )
                ) = position_q.get_mut(predicted_entity) {
                    // Apply command
                    shared_behavior::process_command(
                        &command,
                        &prev_tile_position,
                        &mut next_tile_position,
                        &mut tile_movement,
                    );
                } else {
                    warn!("Could not find components for predicted entity: {:?}", predicted_entity);
                }
            }
        }
    }
}