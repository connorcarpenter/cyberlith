use crate::{components::TileMovement, messages::KeyCommand};

pub fn process_command(tile_movement: &mut TileMovement, key_command: &KeyCommand, prediction: bool) {
    tile_movement.recv_command(key_command, prediction);
}
