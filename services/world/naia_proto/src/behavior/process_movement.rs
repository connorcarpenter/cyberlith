use naia_bevy_shared::Tick;

use crate::{
    behavior::tick_output::TickOutput,
    components::{
        LookDirection, MoveBuffer, PhysicsController, ProcessTickResult, TileMovement,
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
    look_direction_opt: Option<&mut LookDirection>,
    output_opt: Option<&mut TickOutput>,
) -> ProcessTickResult {
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
        tile_movement.process_command(physics, move_buffer, tick, player_command, output_opt);
    }

    let tick_result = tile_movement.process_tick(
        move_buffer.has_buffered_move(),
        physics,
        tick,
        tile_movement_type.is_prediction(),
    );

    if let Some(look_direction) = look_direction_opt {
        if let Some(new_look_direction) = new_look_direction {
            if !tile_movement.is_moving() {
                look_direction.set(new_look_direction);
            }
        }
    }

    return tick_result;
}

pub fn process_result(
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

                // This is important, because client expects to receive the next_tile_position AFTER a simulation step
                physics.step();

                let (dx, dy) = buffered_move_dir.to_delta();

                let next_tile_x = tile_x + dx as i16;
                let next_tile_y = tile_y + dy as i16;

                if let Some(output) = output_opt {
                    output.set_next_tile_position(next_tile_x, next_tile_y);
                    output.set_next_move_buffer(None);
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
