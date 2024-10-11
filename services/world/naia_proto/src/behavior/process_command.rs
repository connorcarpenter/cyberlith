use crate::{components::TileMovement, resources::KeyEvent};

pub fn process_commands(
    tile_movement: &mut TileMovement,
    key_commands: Vec<KeyEvent>,
    prediction: bool,
) {
    tile_movement.recv_command(key_commands, prediction);
}
