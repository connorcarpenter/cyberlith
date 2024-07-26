use crate::{constants::TILE_SIZE, components::{TileMovement, NextTilePosition, PrevTilePosition}, messages::KeyCommand};

pub fn process_command(
    key_command: &KeyCommand,
    prev_tile_position: &PrevTilePosition,
    next_tile_position: &mut NextTilePosition,
    tile_movement: &mut TileMovement,
) {
    if prev_tile_position.x != *next_tile_position.x || prev_tile_position.y != *next_tile_position.y {
        return;
    }

    if key_command.w && !key_command.s {
        *next_tile_position.y = next_tile_position.y.wrapping_sub(1);
    }
    if key_command.s && !key_command.w {
        *next_tile_position.y = next_tile_position.y.wrapping_add(1);
    }
    if key_command.a && !key_command.d {
        *next_tile_position.x = next_tile_position.x.wrapping_sub(1);
    }
    if key_command.d && !key_command.a {
        *next_tile_position.x = next_tile_position.x.wrapping_add(1);
    }

    let x_axis_changed = *next_tile_position.x != prev_tile_position.x;
    let y_axis_changed = *next_tile_position.y != prev_tile_position.y;

    if x_axis_changed || y_axis_changed {

        let distance = if x_axis_changed && y_axis_changed {
            std::f32::consts::SQRT_2
        } else {
            1.0
        };
        tile_movement.next(distance * TILE_SIZE);
    }
}
