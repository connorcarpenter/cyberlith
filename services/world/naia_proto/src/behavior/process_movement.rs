
use crate::{
    components::TileMovement,
};

pub fn process_movement(tile_movement: &mut TileMovement) {
    tile_movement.process_tick();
}