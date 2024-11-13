use naia_bevy_shared::Tick;

use crate::{
    components::{
        MoveBuffer, LookDirection, PhysicsController, ProcessTickResult, TileMovement, TileMovementType,
    },
    messages::PlayerCommands,
};

pub fn process_tick(
    tile_movement_type: TileMovementType,
    tick: Tick,
    player_command: Option<PlayerCommands>,
    has_future: bool,
    tile_movement: &mut TileMovement,
    physics: &mut PhysicsController,
    mut move_buffer_opt: Option<&mut MoveBuffer>,
    look_direction_opt: Option<&mut LookDirection>,
) -> (ProcessTickResult, Option<(i16, i16)>) {
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

    let output = if tile_movement_type.processes_commands() {
        tile_movement.process_command(physics, move_buffer_opt.as_deref_mut(), tick, player_command)
    } else {
        None
    };

    let tick_result = tile_movement.process_tick(has_future, physics);

    if let Some(look_direction) = look_direction_opt {
        if let Some(new_look_direction) = new_look_direction {
            if !tile_movement.is_moving() {
                look_direction.set(new_look_direction);
            }
        }
    }

    return (tick_result, output);
}
