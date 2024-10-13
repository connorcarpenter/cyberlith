use crate::{components::TileMovement, resources::PlayerCommandEvent};

pub fn process_incoming_commands(
    tile_movement: &mut TileMovement,
    player_command_events: Vec<PlayerCommandEvent>,
    prediction: bool,
) {
    tile_movement.recv_command(player_command_events, prediction);
}
