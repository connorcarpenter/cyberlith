
use bevy_ecs::prelude::Component;

use naia_bevy_server::Tick;

use logging::info;

use world_server_naia_proto::{components::{NextTilePosition, TileMovement}};

#[derive(Component)]
pub struct ServerTileMovement {
    tile_movement: TileMovement,
    outbound_next_tile: Option<(i16, i16)>,
}

impl ServerTileMovement {
    pub fn new_stopped(
        next_tile_position: &NextTilePosition,
    ) -> Self {

        let me = Self {
            tile_movement: TileMovement::new_stopped(true, false, next_tile_position),
            outbound_next_tile: None,
        };

        me
    }

    pub fn inner_mut(&mut self) -> &mut TileMovement {
        return &mut self.tile_movement;
    }

    pub fn set_outbound_next_tile(&mut self, outbound_tile_x: i16, outbound_tile_y: i16) {
        self.outbound_next_tile = Some((outbound_tile_x, outbound_tile_y));
    }

    // on the client, never called
    // on the server, called by confirmed entities
    pub fn send_updated_next_tile_position(
        &mut self,
        tick: Tick,
        next_tile_position: &mut NextTilePosition,
    ) {
        if let Some((next_tile_x, next_tile_y)) = self.outbound_next_tile.take() {
            next_tile_position.set(next_tile_x, next_tile_y);

            info!(
                "Send NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
                tick, next_tile_x, next_tile_y
            );
        }
    }
}