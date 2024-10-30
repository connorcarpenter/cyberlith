
use bevy_ecs::prelude::Component;

use game_engine::{naia::Tick, world::{components::{NextTilePosition, TileMovement}}};

#[derive(Component)]
pub struct ClientTileMovement {
    tile_movement: TileMovement,
}

impl ClientTileMovement {
    pub fn new_stopped(
        predicted: bool,
        next_tile_position: &NextTilePosition,
    ) -> Self {

        let me = Self {
            tile_movement: TileMovement::new_stopped(false, predicted, next_tile_position),
        };

        me
    }

    pub fn inner(&self) -> &TileMovement {
        return &self.tile_movement;
    }

    pub fn inner_mut(&mut self) -> &mut TileMovement {
        return &mut self.tile_movement;
    }

    // retrieve the current position of the entity
    pub fn current_position(&self) -> (f32, f32) {
        return self.tile_movement.current_position();
    }

    // on the client, called by confirmed entities
    // on the server, never called
    pub fn recv_updated_next_tile_position(
        &mut self,
        update_tick: Tick,
        next_tile_position: &NextTilePosition,
        prediction: bool,
    ) {
        return self.tile_movement.recv_updated_next_tile_position(update_tick, next_tile_position, prediction);
    }

    // on the client, called by predicted entities
    // on the server, never called
    pub fn recv_rollback(&mut self, server_tile_movement: &TileMovement) {
        return self.tile_movement.recv_rollback(server_tile_movement);
    }
}