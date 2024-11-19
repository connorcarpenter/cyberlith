use bevy_ecs::prelude::Component;

use game_engine::world::components::{MoveBuffer, NextTilePosition, ProcessTickResult, TileMovement};

use crate::components::{client_tile_movement::ClientTileMovement, ConfirmedTileMovement};

#[derive(Component)]
pub struct PredictedTileMovement {
    tile_movement: TileMovement,
    move_buffer: MoveBuffer,
}

impl ClientTileMovement for PredictedTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, &mut MoveBuffer) {
        (&mut self.tile_movement, &mut self.move_buffer)
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
        Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            move_buffer: MoveBuffer::new(),
        }
    }
}

impl From<&ConfirmedTileMovement> for PredictedTileMovement {
    fn from(confirmed: &ConfirmedTileMovement) -> Self {
        let confirmed = confirmed.clone();
        let (tile_movement, move_buffer) = confirmed.decompose_to_values();
        Self {
            tile_movement,
            move_buffer,
        }
    }
}
