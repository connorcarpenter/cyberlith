use bevy_ecs::{system::Res, change_detection::ResMut, event::EventReader, prelude::Query, query::With};

use game_engine::{
    naia::Tick,
    world::{
        components::PhysicsController,
        behavior as shared_behavior, channels::PlayerCommandChannel,
        messages::PlayerCommands, WorldClient, WorldClientTickEvent, WorldServerTickEvent,
    },
};

use crate::{
    components::{AnimationState, ClientTileMovement, Confirmed, Predicted, RenderPosition},
    resources::{Global, TickTracker, InputManager},
};

pub fn client_tick_events(
    mut client: WorldClient,
    global: Res<Global>,
    mut input_manager: ResMut<InputManager>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<(&mut ClientTileMovement, &mut PhysicsController, &mut RenderPosition, &mut AnimationState), With<Predicted>>,
) {
    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted)
    else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        let (
            mut client_tile_movement,
            mut client_physics,
            mut client_render_position,
            mut animation_state
        ) = position_q.get_mut(predicted_entity).unwrap();

        // process commands
        if let Some(outgoing_command) = input_manager.pop_outgoing_command() {

            // outgoing_command.log(client_tick);

            // send command
            // let distance = client_tile_movement.get_dis();
            // info!("Send Command. Tick: {:?}. MoveDir: {:?}, Dis: {:?}", client_tick, outgoing_command.get_move(), distance);
            client.send_tick_buffer_message::<PlayerCommandChannel, PlayerCommands>(&client_tick, &outgoing_command);

            input_manager.save_to_command_history(client_tick, Some(outgoing_command.clone()));
            input_manager.recv_incoming_command(client_tick, Some(outgoing_command));
        } else {
            input_manager.save_to_command_history(client_tick, None);
            input_manager.recv_incoming_command(client_tick, None);
        }

        // process tick
        let player_command = input_manager.pop_incoming_command(client_tick);
        process_tick(
            false,
            false,
            client_tick,
            player_command,
            &mut client_tile_movement,
            &mut client_physics,
            &mut client_render_position,
            &mut animation_state,
        );
    }
}

pub fn process_tick(
    is_server: bool,
    is_rollback: bool,
    tick: Tick,
    player_command: Option<PlayerCommands>,
    tile_movement: &mut ClientTileMovement,
    physics: &mut PhysicsController,
    render_position: &mut RenderPosition,
    animation_state: &mut AnimationState,
) {
    let lookdir_opt = if let Some(player_command) = player_command.as_ref() {
        player_command.get_look()
    } else {
        None
    };

    let (result, _) = shared_behavior::process_tick(tick, player_command, tile_movement.inner_mut(), physics, None);
    tile_movement.process_result(result);

    render_position.recv_position(
        is_server,
        is_rollback,
        physics.position(),
        tick,
    );

    if let Some(lookdir) = lookdir_opt {
        animation_state.recv_lookdir_update(&lookdir);
    }
}

pub fn server_tick_events(
    mut tick_tracker: ResMut<TickTracker>,
    mut tick_reader: EventReader<WorldServerTickEvent>,
    mut position_q: Query<(&mut ClientTileMovement, &mut PhysicsController, &mut RenderPosition, &mut AnimationState), With<Confirmed>>,
) {
    for event in tick_reader.read() {
        let server_tick = event.tick;

        // process movement
        for (mut server_tile_movement, mut server_physics, mut server_render_position, mut animation_state) in position_q.iter_mut() {
            process_tick(
                true,
                false,
                server_tick,
                None,
                &mut server_tile_movement,
                &mut server_physics,
                &mut server_render_position,
                &mut animation_state,
            );
        }

        tick_tracker.set_last_processed_server_tick(server_tick);
    }
}
