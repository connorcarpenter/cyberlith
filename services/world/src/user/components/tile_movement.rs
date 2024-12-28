use bevy_ecs::prelude::Component;

use naia_bevy_server::Tick;

use logging::info;

use world_server_naia_proto::{
    components::{
        MoveBuffer, NetworkedMoveBuffer, NetworkedTileTarget, PhysicsController, TileMovement,
    },
    types::Direction,
};

#[derive(Component)]
pub struct ServerTileMovement {
    tile_movement: TileMovement,
    move_buffer: MoveBuffer,
}

impl ServerTileMovement {
    pub fn new_stopped(next_tile_position: &NetworkedTileTarget) -> Self {
        let me = Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            move_buffer: MoveBuffer::new(),
        };

        me
    }

    pub fn decompose(&mut self) -> (&mut TileMovement, &mut MoveBuffer) {
        (&mut self.tile_movement, &mut self.move_buffer)
    }

    pub fn send_updated_next_tile_position(
        &mut self,
        tick: Tick,
        next_tile_position: &mut NetworkedTileTarget,
        next_tile_x: i16,
        next_tile_y: i16,
        velocity_x: f32,
        velocity_y: f32,
    ) {
        next_tile_position.set(next_tile_x, next_tile_y, velocity_x, velocity_y);

        info!(
            "Send NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
            tick, next_tile_x, next_tile_y
        );
    }

    pub fn send_updated_net_move_buffer(
        &mut self,
        physics: &PhysicsController,
        tick: Tick,
        net_move_buffer: &mut NetworkedMoveBuffer,
        value: Option<Direction>,
    ) {
        net_move_buffer.set(physics, value);

        info!(
            "Send NetworkedMoveBuffer. Tick: {:?}, Value: ({:?})",
            tick, value
        );
    }
}
