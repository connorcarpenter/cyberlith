
use bevy_ecs::prelude::Component;

use game_engine::{
    logging::{info, warn},
    naia::Tick,
    world::{
        components::{NextTilePosition, MoveBuffer, ProcessTickResult, TileMovement},
        types::Direction,
    },
};

use crate::components::{future_tile_buffer::FutureTileBuffer, client_tile_movement::ClientTileMovement};

#[derive(Component, Clone)]
pub struct ConfirmedTileMovement {
    pub(crate) tile_movement: TileMovement,
    pub(crate) future_tile_buffer: FutureTileBuffer,
}

impl ClientTileMovement for ConfirmedTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, Option<&mut MoveBuffer>) {
        (&mut self.tile_movement, None)
    }

    fn process_result(&mut self, result: ProcessTickResult) {

        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                if self.future_tile_buffer.has_tiles() {
                    self.future_tile_buffer.pop_and_use_buffered_future_tiles(&mut self.tile_movement, tile_x, tile_y);
                } else {
                    self.tile_movement.set_stopped(tile_x, tile_y);
                }
            }
            ProcessTickResult::DoNothing => {}
        }
    }
}

impl ConfirmedTileMovement {
    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        let me = Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            future_tile_buffer: FutureTileBuffer::new(),
        };

        me
    }

    // called by confirmed entities
    pub fn recv_updated_next_tile_position(
        &mut self,
        update_tick: Tick,
        next_tile_position: &NextTilePosition,
    ) {
        let (next_tile_x, next_tile_y) = (next_tile_position.x(), next_tile_position.y());
        info!(
            "Recv NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
            update_tick, next_tile_x, next_tile_y
        );

        let (current_tile_x, current_tile_y) = self.tile_movement.tile_position();

        if self.tile_movement.is_stopped() {
            // is stopped

            if current_tile_x == next_tile_x && current_tile_y == next_tile_y {
                return;
            }

            let dx = (next_tile_x - current_tile_x) as i8;
            let dy = (next_tile_y - current_tile_y) as i8;

            if let Some(move_dir) = Direction::from_delta(dx, dy) {
                self.tile_movement.set_moving(move_dir);
            } else {
                warn!(
                    "Invalid move direction. Prev: ({:?}, {:?}), Next: ({:?}, {:?}). Buffering...",
                    current_tile_x, current_tile_y, next_tile_x, next_tile_y
                );

                // buffering the invalid position, which will add pathfinding tiles to the buffer
                self.future_tile_buffer.buffer_updated_next_tile_position(
                    update_tick,
                    current_tile_x,
                    current_tile_y,
                    next_tile_position.x(),
                    next_tile_position.y(),
                );

                // pop buffer
                self.future_tile_buffer.pop_and_use_buffered_future_tiles(&mut self.tile_movement, current_tile_x, current_tile_y);
            }
        } else {
            // is moving
            self.future_tile_buffer.buffer_updated_next_tile_position(
                update_tick,
                current_tile_x,
                current_tile_y,
                next_tile_position.x(),
                next_tile_position.y(),
            );
        }
    }
}

