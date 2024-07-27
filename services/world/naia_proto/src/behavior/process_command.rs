use naia_bevy_shared::Tick;

use crate::{
    components::TileMovement,
    messages::KeyCommand,
};

pub fn process_command(
    tile_movement: &mut TileMovement,
    tick: Tick,
    key_command: &KeyCommand,
) {
    tile_movement.recv_command(tick, key_command);
}
