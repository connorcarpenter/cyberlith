use bevy_ecs::prelude::Component;

use game_engine::world::components::{MoveBuffer, NextTilePosition, ProcessTickResult, TileMovement};

use crate::components::{client_tile_movement::ClientTileMovement, ConfirmedTileMovement};

#[derive(Component)]
pub struct PredictedTileMovement {
    tile_movement: TileMovement,
    move_buffer: MoveBuffer,
}

impl ClientTileMovement for PredictedTileMovement {
    fn inner_mut(&mut self) -> (&mut TileMovement, Option<&mut MoveBuffer>) {
        (&mut self.tile_movement, Some(&mut self.move_buffer))
    }

    fn process_result(&mut self, result: ProcessTickResult) {

        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                if self.move_buffer.has_buffered_move() {
                    let buffered_move_dir = self.move_buffer.pop_buffered_move().unwrap();
                    self.tile_movement.set_continue(tile_x, tile_y, buffered_move_dir);
                } else {
                    self.tile_movement.set_stopped(tile_x, tile_y);
                }
            },
            ProcessTickResult::DoNothing => {}
        }
    }
}

impl PredictedTileMovement {
    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        let me = Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            move_buffer: MoveBuffer::new(),
        };

        me
    }

    // called by predicted entities
    pub fn recv_rollback(&mut self, confirmed_tile_movement: &ConfirmedTileMovement) {
        self.tile_movement
            .mirror(&confirmed_tile_movement.tile_movement);
    }
}
