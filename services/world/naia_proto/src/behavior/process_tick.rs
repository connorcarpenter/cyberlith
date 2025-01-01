use naia_bevy_shared::Tick;

use crate::{
    behavior::tick_output::TickOutput,
    components::{
        NetworkedLookDir, MoveBuffer, PhysicsController, ProcessTickResult, TileMovement,
        TileMovementType,
    },
    messages::PlayerCommands,
};

pub fn process_tick(
    tile_movement_type: TileMovementType,
    tick: Tick,
    player_command: Option<PlayerCommands>,
    tile_movement: &mut TileMovement,
    physics: &mut PhysicsController,
    move_buffer: &mut MoveBuffer,
    look_direction_opt: Option<&mut NetworkedLookDir>,
    mut output_opt: Option<&mut TickOutput>,
) {
    let new_look_direction = {
        if look_direction_opt.is_none() {
            None
        } else {
            if let Some(player_command) = player_command.as_ref() {
                player_command.get_look()
            } else {
                None
            }
        }
    };

    if tile_movement_type.processes_commands() {
        let output_opt: Option<&mut TickOutput> = output_opt.as_mut().map(
            |output| {
                let output: &mut TickOutput = output;
                output
            }
        );
        process_command(tile_movement, physics, move_buffer, tick, player_command, output_opt);
    }

    let tick_result = tile_movement.process_tick(
        tick,
        tile_movement_type.is_prediction(),
        physics,
        move_buffer.buffered_move(),
    );

    if let Some(look_direction) = look_direction_opt {
        if let Some(new_look_direction) = new_look_direction {
            if !tile_movement.is_moving() {
                look_direction.set(new_look_direction);
            }
        }
    }

    process_result(tile_movement, move_buffer, physics, tick_result, output_opt);

    physics.step();
}

// on the client, called by predicted entities
// on the server, called by confirmed entities
fn process_command(
    tile_movement: &mut TileMovement,
    physics: &PhysicsController,
    move_buffer: &mut MoveBuffer,
    tick: Tick,
    command: Option<PlayerCommands>,
    output_opt: Option<&mut TickOutput>,
) {
    let Some(command) = command else {
        return;
    };
    let Some(direction) = command.get_move() else {
        return;
    };

    // info!("process_command: {:?} {:?}", tick, direction);

    if tile_movement.is_stopped() {
        let state = tile_movement.as_stopped_mut();
        let (tile_x, tile_y) = state.tile_position();
        let (dx, dy) = direction.to_delta();

        let next_tile_x = tile_x + dx as i16;
        let next_tile_y = tile_y + dy as i16;

        tile_movement.set_moving(direction);

        if let Some(tick_output) = output_opt {
            tick_output.set_net_tile_target(next_tile_x, next_tile_y);
        }
        return;
    } else {
        let state = tile_movement.as_moving_mut();
        if state.can_buffer_movement(physics) {
            let prev_move = move_buffer.buffered_move();

            state.buffer_movement(move_buffer, tick, direction);

            if prev_move != Some(direction) {
                if let Some(tick_output) = output_opt {
                    tick_output.set_net_move_buffer(Some(direction));
                }
                return;
            }
        }

        return;
    }
}

fn process_result(
    tile_movement: &mut TileMovement,
    move_buffer: &mut MoveBuffer,
    physics: &mut PhysicsController,
    result: ProcessTickResult,
    output_opt: Option<&mut TickOutput>,
) {
    match result {
        ProcessTickResult::ShouldStop(tile_x, tile_y) => {
            if move_buffer.has_buffered_move() {
                let buffered_move_dir = move_buffer.pop_buffered_move().unwrap();

                tile_movement.set_continue(tile_x, tile_y, buffered_move_dir);

                let (dx, dy) = buffered_move_dir.to_delta();

                let next_tile_x = tile_x + dx as i16;
                let next_tile_y = tile_y + dy as i16;

                if let Some(output) = output_opt {
                    output.set_net_tile_target(next_tile_x, next_tile_y);
                    output.set_net_move_buffer(None);
                }

                return;
            } else {
                tile_movement.set_stopped(tile_x, tile_y);
                physics.set_velocity(0.0, 0.0, false);
            }
        }
        ProcessTickResult::DoNothing => {}
    }
}
