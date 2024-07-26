use crate::{constants::TILE_SIZE, components::{NextTilePosition, Position, PrevTilePosition, TileMovement}};

pub fn process_movement(
    prev_tile_position: &mut PrevTilePosition,
    next_tile_position: &NextTilePosition,
    tile_movement: &mut TileMovement,
    position: &mut Position,
) {
    if prev_tile_position.x == *next_tile_position.x && prev_tile_position.y == *next_tile_position.y {
        return;
    }

    if tile_movement.complete() {
        return;
    }

    tile_movement.tick();
    if tile_movement.complete() {
        prev_tile_position.x = *next_tile_position.x;
        prev_tile_position.y = *next_tile_position.y;
        position.x = *next_tile_position.x as f32 * TILE_SIZE;
        position.y = *next_tile_position.y as f32 * TILE_SIZE;
    } else {
        let interp = tile_movement.interp();
        let prev_x = prev_tile_position.x as f32;
        let prev_y = prev_tile_position.y as f32;
        position.x = (((*next_tile_position.x as f32 - prev_x) * interp) + prev_x) * TILE_SIZE;
        position.y = (((*next_tile_position.y as f32 - prev_y) * interp) + prev_y) * TILE_SIZE;
    }
}