use crate::{components::TileMovement, messages::CommandReadState};

pub fn process_commands(
    tile_movement: &mut TileMovement,
    key_commands: Vec<CommandReadState>,
    prediction: bool,
) {
    tile_movement.recv_command(key_commands, prediction);
}
