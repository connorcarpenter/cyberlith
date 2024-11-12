
use bevy_ecs::prelude::Component;

use game_engine::{world::{components::{NextTilePosition, ProcessTickResult, TileMovement}}};

use crate::components::{ConfirmedTileMovement, client_tile_movement::ClientTileMovement};

#[derive(Component)]
pub struct PredictedTileMovement {
    tile_movement: TileMovement,
}

impl ClientTileMovement for PredictedTileMovement {
    fn inner_mut(&mut self) -> &mut TileMovement {
        return &mut self.tile_movement;
    }

    fn process_result(&mut self, result: ProcessTickResult) {
        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                self.tile_movement.set_stopped(tile_x, tile_y);
            }
            ProcessTickResult::DoNothing => {},
            ProcessTickResult::ShouldContinue(_, _, _) => {
                panic!("ShouldContinue not expected");
            }
        }
    }
}

impl PredictedTileMovement {
    pub fn new_stopped(
        next_tile_position: &NextTilePosition,
    ) -> Self {

        let me = Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
        };

        me
    }

    // called by predicted entities
    pub fn recv_rollback(&mut self, confirmed_tile_movement: &ConfirmedTileMovement) {
        self.tile_movement.mirror(&confirmed_tile_movement.tile_movement);
    }
}