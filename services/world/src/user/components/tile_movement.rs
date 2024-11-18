use bevy_ecs::prelude::Component;

use naia_bevy_server::Tick;

use logging::info;

use world_server_naia_proto::components::{HasMoveBuffered, MoveBuffer, NextTilePosition, ProcessTickResult, TileMovement};
use world_server_naia_proto::types::Direction;

#[derive(Component)]
pub struct ServerTileMovement {
    tile_movement: TileMovement,
    move_buffer: MoveBuffer,
}

impl ServerTileMovement {
    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        let me = Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            move_buffer: MoveBuffer::new(),
        };

        me
    }

    pub fn decompose(&mut self) -> (&mut TileMovement, &mut MoveBuffer) {
        (&mut self.tile_movement, &mut self.move_buffer)
    }

    pub fn process_result(&mut self, result: ProcessTickResult) -> (Option<(i16, i16)>, Option<Option<Direction>>) {

        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                if self.move_buffer.has_buffered_move() {
                    let buffered_move_dir = self.move_buffer.pop_buffered_move().unwrap();

                    self.tile_movement.set_continue(tile_x, tile_y, buffered_move_dir);

                    let (dx, dy) = buffered_move_dir.to_delta();

                    let next_tile_x = tile_x + dx as i16;
                    let next_tile_y = tile_y + dy as i16;

                    return (Some((next_tile_x, next_tile_y)), Some(None));
                } else {
                    self.tile_movement.set_stopped(tile_x, tile_y);
                }
            },
            ProcessTickResult::DoNothing => {}
        }

        return (None, None);
    }

    pub fn has_future(&self) -> bool {
        self.move_buffer.has_buffered_move()
    }

    pub fn send_updated_next_tile_position(
        &mut self,
        tick: Tick,
        next_tile_position: &mut NextTilePosition,
        next_tile_x: i16,
        next_tile_y: i16,
    ) {
        next_tile_position.set(next_tile_x, next_tile_y);

        info!(
            "Send NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
            tick, next_tile_x, next_tile_y
        );
    }

    pub fn send_updated_has_move_buffered(
        &mut self,
        tick: Tick,
        has_move_buffered: &mut HasMoveBuffered,
        value: Option<Direction>,
    ) {
        has_move_buffered.set_buffered(value);

        info!(
            "Send HasMoveBuffered. Tick: {:?}, Value: ({:?})",
            tick, value
        );
    }
}
