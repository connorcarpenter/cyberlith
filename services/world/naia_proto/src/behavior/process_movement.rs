use naia_bevy_shared::Tick;

use crate::{
    components::TileMovement,
};

pub fn process_movement(
    current_tick: Tick,
    tile_movement: &mut TileMovement,
) {
    tile_movement.process_tick(current_tick);
}