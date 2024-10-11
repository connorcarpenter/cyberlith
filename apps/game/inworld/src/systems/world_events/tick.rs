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
    let command_opt = input_manager.take_queued_command();

    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted)
    else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        let (mut client_tile_movement, mut client_render_position) =
            position_q.get_mut(predicted_entity).unwrap();

        // process commands
        let Some(command) = command_opt.as_ref() else {
            continue;
        };

        // save to command history
        input_manager.save_to_command_history(client_tick, command);

        shared_behavior::process_command(&mut client_tile_movement, command, true);

        // process tick
        process_tick(
            false,
            false,
            client_tick,
            &mut client_tile_movement,
            &mut client_render_position,
        );

        // send command
        client.send_tick_buffer_message::<PlayerCommandChannel, KeyCommand>(&client_tick, command);
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
