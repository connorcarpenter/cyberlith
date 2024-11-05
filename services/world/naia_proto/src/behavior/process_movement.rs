use naia_bevy_shared::Tick;

use crate::{messages::PlayerCommands, components::{TileMovement, ProcessTickResult, LookDirection}};

pub fn process_tick(
    tick: Tick,
    player_command: Option<PlayerCommands>,
    tile_movement: &mut TileMovement,
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

    let (result, output) = tile_movement.process_tick(tick, player_command);

    if let Some(look_direction) = look_direction_opt {
        if let Some(new_look_direction) = new_look_direction {
            if !tile_movement.is_moving() {
                look_direction.set(new_look_direction);
            }
        }
    }

    return (result, output);
}
