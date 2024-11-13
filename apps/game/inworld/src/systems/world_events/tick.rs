use bevy_ecs::{
    change_detection::ResMut, event::EventReader, prelude::Query, query::With, system::Res,
};

use game_engine::{
    naia::Tick,
    world::{
        behavior as shared_behavior,
        channels::PlayerCommandChannel,
        components::{PhysicsController, TileMovementType},
        messages::PlayerCommands,
        WorldClient, WorldClientTickEvent, WorldServerTickEvent,
    },
};

use crate::{
    components::{
        AnimationState, ClientTileMovement, Confirmed, ConfirmedTileMovement, Predicted,
        PredictedTileMovement, RenderPosition,
    },
    resources::{Global, InputManager, TickTracker},
};

pub fn client_tick_events(
    mut client: WorldClient,
    global: Res<Global>,
    mut input_manager: ResMut<InputManager>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<
        (
            &mut PredictedTileMovement,
            &mut PhysicsController,
            &mut RenderPosition,
            &mut AnimationState,
        ),
        With<Predicted>,
    >,
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
            mut animation_state,
        ) = position_q.get_mut(predicted_entity).unwrap();

        // process commands
        if let Some(outgoing_command) = input_manager.pop_outgoing_command() {
            // outgoing_command.log(client_tick);

            // send command
            // let distance = client_tile_movement.get_dis();
            // info!("Send Command. Tick: {:?}. MoveDir: {:?}, Dis: {:?}", client_tick, outgoing_command.get_move(), distance);
            client.send_tick_buffer_message::<PlayerCommandChannel, PlayerCommands>(
                &client_tick,
                &outgoing_command,
            );

            input_manager.save_to_command_history(client_tick, Some(outgoing_command.clone()));
            input_manager.recv_incoming_command(client_tick, Some(outgoing_command));
        } else {
            input_manager.save_to_command_history(client_tick, None);
            input_manager.recv_incoming_command(client_tick, None);
        }

        // process tick
        let player_command = input_manager.pop_incoming_command(client_tick);
        let client_tile_movement_2: &mut PredictedTileMovement = &mut client_tile_movement;
        process_tick(
            TileMovementType::ClientPredicted,
            client_tick,
            player_command,
            client_tile_movement_2,
            &mut client_physics,
            &mut client_render_position,
            &mut animation_state,
        );
    }
}

pub fn process_tick(
    tile_movement_type: TileMovementType,
    tick: Tick,
    player_command: Option<PlayerCommands>,
    tile_movement: &mut dyn ClientTileMovement,
    physics: &mut PhysicsController,
    render_position: &mut RenderPosition,
    animation_state: &mut AnimationState,
) {
    let lookdir_opt = if let Some(player_command) = player_command.as_ref() {
        player_command.get_look()
    } else {
        None
    };

    let has_future = tile_movement.has_future();
    let (inner_tile_movement, inner_move_buffer_opt) = tile_movement.decompose();
    let (result, _, _) = shared_behavior::process_tick(
        tile_movement_type,
        tick,
        player_command,
        has_future,
        inner_tile_movement,
        physics,
        inner_move_buffer_opt,
        None,
    );
    tile_movement.process_result(result);

    render_position.recv_position(physics.position(), tick);

    if let Some(lookdir) = lookdir_opt {
        animation_state.recv_lookdir_update(&lookdir);
    }
}

pub fn server_tick_events(
    mut tick_tracker: ResMut<TickTracker>,
    mut tick_reader: EventReader<WorldServerTickEvent>,
    mut position_q: Query<
        (
            &mut ConfirmedTileMovement,
            &mut PhysicsController,
            &mut RenderPosition,
            &mut AnimationState,
        ),
        With<Confirmed>,
    >,
) {
    for event in tick_reader.read() {
        let server_tick = event.tick;

        // process movement
        for (
            mut confirmed_tile_movement,
            mut confirmed_physics,
            mut confirmed_render_position,
            mut confirmed_animation_state,
        ) in position_q.iter_mut()
        {
            let confirmed_tile_movement_2: &mut ConfirmedTileMovement =
                &mut confirmed_tile_movement;
            process_tick(
                TileMovementType::ClientConfirmed,
                server_tick,
                None,
                confirmed_tile_movement_2,
                &mut confirmed_physics,
                &mut confirmed_render_position,
                &mut confirmed_animation_state,
            );
        }

        tick_tracker.set_last_processed_server_tick(server_tick);
    }
}
