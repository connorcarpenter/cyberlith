
use bevy_ecs::prelude::Component;

use naia_bevy_server::Tick;

use world_server_naia_proto::{components::{NextTilePosition, TileMovement}};

#[derive(Component)]
pub struct ServerTileMovement {
    tile_movement: TileMovement,
}

impl ServerTileMovement {
    pub fn new_stopped(
        next_tile_position: &NextTilePosition,
    ) -> Self {

        let me = Self {
            tile_movement: TileMovement::new_stopped(true, false, next_tile_position),
        };

        me
    }

    pub fn inner_mut(&mut self) -> &mut TileMovement {
        return &mut self.tile_movement;
    }

    // on the client, never called
    // on the server, called by confirmed entities
    pub fn send_updated_next_tile_position(
        &mut self,
        tick: Tick,
        next_tile_position: &mut NextTilePosition,
    ) {
        return self.tile_movement.send_updated_next_tile_position(tick, next_tile_position);
    }
}