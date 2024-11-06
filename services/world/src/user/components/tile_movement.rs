
use bevy_ecs::prelude::Component;

use naia_bevy_server::Tick;

use logging::info;

use world_server_naia_proto::{components::{NextTilePosition, TileMovement}};
use world_server_naia_proto::components::ProcessTickResult;

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

    // pub fn get_dis(&self) -> f32 {
    //     return self.tile_movement.get_dis();
    // }

    pub fn process_result(&mut self, result: ProcessTickResult) {
        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                self.tile_movement.set_stopped(tile_x, tile_y);
            }
            ProcessTickResult::DoNothing => {},
            ProcessTickResult::ShouldContinue(_, _, _) => {
                panic!("ShouldMove not expected");
            }
        }
    }

    // on the client, never called
    // on the server, called by confirmed entities
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
}