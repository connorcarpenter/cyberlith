use naia_bevy_shared::Tick;

use crate::{
    components::{Position, PrevTilePosition, TileMovement},
    constants::TILE_SIZE,
};

pub fn process_movement(
    prev_tile_position: &mut PrevTilePosition,
    next_tile_position_x: i16,
    next_tile_position_y: i16,
    tile_movement: &mut TileMovement,
    position: &mut Position,
    tick: Tick,
) {
    let next_position_x_f32 = next_tile_position_x as f32 * TILE_SIZE;
    let next_position_y_f32 = next_tile_position_y as f32 * TILE_SIZE;

    if tile_movement.complete()
        || (prev_tile_position.x == next_tile_position_x
            && prev_tile_position.y == next_tile_position_y)
    {
        tile_movement.process_tick(tick);
        position.set(tick, next_position_x_f32, next_position_y_f32);
        return;
    }

    tile_movement.process_tick(tick);

    if tile_movement.complete() {
        prev_tile_position.x = next_tile_position_x;
        prev_tile_position.y = next_tile_position_y;
        position.set(tick, next_position_x_f32, next_position_y_f32);
    } else {
        let interp = tile_movement.interp();
        let prev_position_x_f32 = prev_tile_position.x as f32 * TILE_SIZE;
        let prev_position_y_f32 = prev_tile_position.y as f32 * TILE_SIZE;

        position.set(
            tick,
            ((next_position_x_f32 - prev_position_x_f32) * interp) + prev_position_x_f32,
            ((next_position_y_f32 - prev_position_y_f32) * interp) + prev_position_y_f32,
        );
    }
}
