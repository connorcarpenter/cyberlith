
use game_engine::{
    world::{
        components::{MoveBuffer, ProcessTickResult, TileMovement},
    },
};

use crate::components::{client_tile_movement::ClientTileMovement, ConfirmedTileMovement, PredictedTileMovement};

pub enum RollbackTileMovement {
    Confirmed(ConfirmedTileMovement),
    Predicted(PredictedTileMovement),
}

impl ClientTileMovement for RollbackTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, Option<&mut MoveBuffer>) {
        match self {
            Self::Confirmed(confirmed_tile_movement) => confirmed_tile_movement.decompose(),
            Self::Predicted(predicted_tile_movement) => predicted_tile_movement.decompose(),
        }
    }

    fn process_result(&mut self, result: ProcessTickResult) {
        match self {
            Self::Confirmed(confirmed_tile_movement) => {
                confirmed_tile_movement.process_result(result);
                if !confirmed_tile_movement.future_tile_buffer.has_tiles() {
                    let tile_movement = confirmed_tile_movement.tile_movement.clone();
                    *self = Self::Predicted(PredictedTileMovement::from_tile_movement(tile_movement));
                }
            },
            Self::Predicted(predicted_tile_movement) => predicted_tile_movement.process_result(result),
        }
    }
}

impl From<ConfirmedTileMovement> for RollbackTileMovement {
    fn from(confirmed_tile_movement: ConfirmedTileMovement) -> Self {
        Self::Confirmed(confirmed_tile_movement)
    }
}

impl Into<PredictedTileMovement> for RollbackTileMovement {
    fn into(self) -> PredictedTileMovement {
        match self {
            Self::Confirmed(confirmed_tile_movement) => {
                let tile_movement = confirmed_tile_movement.tile_movement;
                PredictedTileMovement::from_tile_movement(tile_movement)
            },
            Self::Predicted(predicted_tile_movement) => predicted_tile_movement,
        }
    }
}

