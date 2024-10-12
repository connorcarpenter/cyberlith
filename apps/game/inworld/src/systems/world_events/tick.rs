use bevy_ecs::{system::Res, change_detection::ResMut, event::EventReader, prelude::Query, query::With};

use game_engine::{
    naia::Tick,
    world::{
        behavior as shared_behavior, channels::PlayerCommandChannel, components::TileMovement,
        messages::KeyCommand, WorldClient, WorldClientTickEvent, WorldServerTickEvent,
    },
};

use crate::{
    components::{Confirmed, Predicted, RenderPosition},
    resources::{Global, InputManager},
};

pub fn client_tick_events(
    mut client: WorldClient,
    global: Res<Global>,
    mut input_manager: ResMut<InputManager>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<(&mut TileMovement, &mut RenderPosition), With<Predicted>>,
) {
    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted)
    else {
        return;
    };

    let Some(client_instant) = client.client_instant() else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        let (mut client_tile_movement, mut client_render_position) =
            position_q.get_mut(predicted_entity).unwrap();

        // process commands
        if let Some(outgoing_command) = input_manager.pop_outgoing_command(client_instant) {

            // command.log(client_tick);

            // save to command history
            input_manager.save_to_command_history(client_tick, &outgoing_command);

            // send command
            client.send_tick_buffer_message::<PlayerCommandChannel, KeyCommand>(&client_tick, &outgoing_command);

            input_manager.recv_incoming_command(client_tick, Some(outgoing_command));
        } else {
            input_manager.recv_incoming_command(client_tick, None);
        }

        let incoming_commands = input_manager.pop_incoming_commands(client_tick);
        shared_behavior::process_incoming_commands(&mut client_tile_movement, incoming_commands, true);

        // process tick
        process_tick(
            false,
            false,
            client_tick,
            &mut client_tile_movement,
            &mut client_render_position,
        );
    }
}

pub fn process_tick(
    is_server: bool,
    is_rollback: bool,
    tick: Tick,
    mut tile_movement: &mut TileMovement,
    render_position: &mut RenderPosition,
) {
    shared_behavior::process_movement(&mut tile_movement);

    render_position.recv_position(
        is_server,
        is_rollback,
        tile_movement.current_position(),
        tick,
    );
}

pub fn server_tick_events(
    mut tick_reader: EventReader<WorldServerTickEvent>,
    mut position_q: Query<(&mut TileMovement, &mut RenderPosition), With<Confirmed>>,
) {
    for event in tick_reader.read() {
        let server_tick = event.tick;

        // process movement
        for (mut server_tile_movement, mut server_render_position) in position_q.iter_mut() {
            process_tick(
                true,
                false,
                server_tick,
                &mut server_tile_movement,
                &mut server_render_position,
            );
        }
    }
}
