use bevy_ecs::{change_detection::ResMut, event::EventReader, prelude::Query, query::With};

use game_engine::{
    logging::warn,
    world::{
        behavior as shared_behavior,
        channels::PlayerCommandChannel,
        components::{NextTilePosition, TileMovement},
        messages::KeyCommand,
        WorldClient, WorldClientTickEvent, WorldServerTickEvent,
    },
};

use crate::{
    components::{Confirmed, Predicted},
    resources::Global,
};

pub fn client_tick_events(
    mut client: WorldClient,
    mut global: ResMut<Global>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<
        &mut TileMovement,
        With<Predicted>,
    >,
) {
    let command_opt = global.queued_command.take();

    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted)
    else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        let mut tile_movement = position_q.get_mut(predicted_entity).unwrap();

        // process commands
        let Some(command) = command_opt.as_ref() else {
            continue;
        };

        // save to command history
        {
            if !global.command_history.can_insert(&client_tick) {
                // History is full, should this be possible??
                panic!(
                    "Command History is full, cannot insert command for tick: {:?}",
                    client_tick
                );
            }

            // Record command
            global.command_history.insert(client_tick, command.clone());
        }

        shared_behavior::process_command(
            &mut tile_movement,
            command,
        );

        // process movement
        shared_behavior::process_movement(&mut tile_movement);

        // send command
        client.send_tick_buffer_message::<PlayerCommandChannel, KeyCommand>(&client_tick, command);
    }
}

pub fn server_tick_events(
    mut tick_reader: EventReader<WorldServerTickEvent>,
    mut position_q: Query<&mut TileMovement, With<Confirmed>>,
) {
    for event in tick_reader.read() {
        let server_tick = event.tick;

        // process movement
        for mut tile_movement in position_q.iter_mut()
        {
            shared_behavior::process_movement(&mut tile_movement);
        }
    }
}
