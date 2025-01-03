use bevy_ecs::prelude::Component;

use game_app_network::world::components::{MoveBuffer, NetworkedTileTarget, TileMovement};

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
}

impl PredictedTileMovement {
    pub fn new_stopped(net_tile_target: &NetworkedTileTarget) -> Self {
        Self {
            tile_movement: TileMovement::new_stopped(net_tile_target),
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
