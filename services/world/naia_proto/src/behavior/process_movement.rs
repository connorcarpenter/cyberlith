use naia_bevy_shared::Tick;

use crate::{constants::TILE_SIZE, components::{Position, PrevTilePosition, TileMovement}};

pub fn process_movement(
    prev_tile_position: &mut PrevTilePosition,
    next_tile_position_x: i16,
    next_tile_position_y: i16,
    tile_movement: &mut TileMovement,
    position: &mut Position,
    tick: Tick,
) {
    if prev_tile_position.x == next_tile_position_x && prev_tile_position.y == next_tile_position_y {
        position.set(tick, next_tile_position_x as f32 * TILE_SIZE, next_tile_position_y as f32 * TILE_SIZE);
        return;
    }

    if tile_movement.complete() {
        tile_movement.process_tick(tick);
        position.set(tick, next_tile_position_x as f32 * TILE_SIZE, next_tile_position_y as f32 * TILE_SIZE);
        return;
    }

    tile_movement.process_tick(tick);
    if tile_movement.complete() {
        prev_tile_position.x = next_tile_position_x;
        prev_tile_position.y = next_tile_position_y;
        position.set(tick, next_tile_position_x as f32 * TILE_SIZE, next_tile_position_y as f32 * TILE_SIZE);
    } else {
        let interp = tile_movement.interp();
        let prev_x = prev_tile_position.x as f32;
        let prev_y = prev_tile_position.y as f32;
        position.set(tick, (((next_tile_position_x as f32 - prev_x) * interp) + prev_x) * TILE_SIZE, (((next_tile_position_y as f32 - prev_y) * interp) + prev_y) * TILE_SIZE);
    }
}