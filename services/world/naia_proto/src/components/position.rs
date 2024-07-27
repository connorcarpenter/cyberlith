use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate, Tick};

use crate::messages::KeyCommand;

// This is networked

#[derive(Component, Replicate)]
pub struct NextTilePosition {
    x: Property<i16>,
    y: Property<i16>,
}

impl NextTilePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_complete(x, y)
    }

    pub fn x(&self) -> i16 {
        *self.x
    }

    pub fn y(&self) -> i16 {
        *self.y
    }

    pub fn set_x(&mut self, x: i16) {
        *self.x = x;
    }

    pub fn set_y(&mut self, y: i16) {
        *self.y = y;
    }

    pub fn set(&mut self, x: i16, y: i16) {
        *self.x = x;
        *self.y = y;
    }
}

// These are not networked

#[derive(Component)]
pub struct TileMovement {
    state: TileMovementState,
}

impl TileMovement {

    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        Self {
            state: TileMovementState::stopped(),
        }
    }

    pub fn stopped() -> Self {
        Self {
            state: TileMovementState::stopped(),
        }
    }

    pub fn moving() -> Self {
        Self {
            state: TileMovementState::moving(),
        }
    }

    // retrieve the current position of the entity
    pub fn current_position(&self) -> (f32, f32) {
        todo!();
    }

    // on the client, called by predicted entities
    // on the server, called by confirmed entities
    pub fn recv_command(&mut self, key_command: &KeyCommand) {
        todo!();
    }

    // on the client, called by confirmed entities
    // on the server, never called
    pub fn recv_updated_next_tile_position(&mut self, next_tile_position: &NextTilePosition, update_tick: Tick) {
        todo!();
    }

    // on the client, never called
    // on the server, called by confirmed entities
    pub fn send_updated_next_tile_position(&mut self, next_tile_position: &mut NextTilePosition) {
        todo!();
    }

    // on the client, called by predicted entities
    // on the server, never called
    pub fn recv_rollback(&mut self, server_tile_movement: &TileMovement) {
        todo!();
    }

    // call on each tick
    pub fn process_movement(&mut self) {
        todo!();
    }
}

// Tile Movement State
enum TileMovementState {
    Stopped(TileMovementStoppedState),
    Moving(TileMovementMovingState),
}

impl TileMovementState {
    fn stopped() -> Self {
        Self::Stopped(TileMovementStoppedState::new())
    }

    fn moving() -> Self {
        Self::Moving(TileMovementMovingState::new())
    }
}

// Tile Movement Stopped State
struct TileMovementStoppedState {
}

impl TileMovementStoppedState {
    fn new() -> Self {
        Self {
        }
    }
}

// Tile Movement Moving State
struct TileMovementMovingState {
}

impl TileMovementMovingState {
    fn new() -> Self {
        Self {
        }
    }
}