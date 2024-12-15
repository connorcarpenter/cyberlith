use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Serde, Property, Replicate, SignedVariableInteger};
use math::Vec2;

use crate::{constants::TILE_SIZE, types::Direction, components::PhysicsController};

// This is networked

#[derive(Component, Replicate)]
pub struct NetworkedMoveBuffer {
    buffered: Property<Option<BufferedMove>>,
}

impl NetworkedMoveBuffer {
    pub fn new() -> Self {
        Self::new_complete(None)
    }

    // returns Option<(BufferedMoveDir, Position, Velocity)>
    pub fn get(&self) -> Option<(Direction, Vec2, Vec2)> {
        if self.buffered.is_none() {
            return None;
        }
        Some(self.buffered.as_ref().unwrap().get())
    }

    pub fn set(
        &mut self,
        physics: &PhysicsController,
        buffered: Option<Direction>
    ) {
        if buffered.is_none() {
            *self.buffered = None;
            return;
        }

        let move_dir = buffered.unwrap();
        *self.buffered = Some(BufferedMove::new(physics, move_dir));
    }
}

#[derive(Serde, PartialEq, Clone)]
struct BufferedMove {
    direction: Direction,
    tile_x: i16,
    tile_y: i16,
    position_delta_x: SignedVariableInteger<14>,
    position_delta_y: SignedVariableInteger<14>,
    velocity_x: SignedVariableInteger<11>,
    velocity_y: SignedVariableInteger<11>,
}

impl BufferedMove {
    fn new(physics: &PhysicsController, direction: Direction) -> Self {

        let position = physics.position();
        let tile_x = (position.x / TILE_SIZE).round() as i16;
        let tile_y = (position.y / TILE_SIZE).round() as i16;
        let position_delta_x = position.x - (tile_x as f32 * TILE_SIZE);
        let position_delta_y = position.y - (tile_y as f32 * TILE_SIZE);
        let position_delta_x = SignedVariableInteger::new((position_delta_x * 100.0) as i128);
        let position_delta_y = SignedVariableInteger::new((position_delta_y * 100.0) as i128);

        let velocity = physics.velocity();
        let velocity_x = SignedVariableInteger::new((velocity.x * 100.0) as i128);
        let velocity_y = SignedVariableInteger::new((velocity.y * 100.0) as i128);

        Self {
            direction,
            tile_x,
            tile_y,
            position_delta_x,
            position_delta_y,
            velocity_x,
            velocity_y,
        }
    }

    fn get(&self) -> (Direction, Vec2, Vec2) {
        let tile_position_x = self.tile_x as f32 * TILE_SIZE;
        let tile_position_y = self.tile_y as f32 * TILE_SIZE;
        let position_delta_x = self.position_delta_x.get() as f32 / 100.0;
        let position_delta_y = self.position_delta_y.get() as f32 / 100.0;
        let position = Vec2::new(tile_position_x + position_delta_x, tile_position_y + position_delta_y);

        let velocity_x = self.velocity_x.get() as f32 / 100.0;
        let velocity_y = self.velocity_y.get() as f32 / 100.0;
        let velocity = Vec2::new(velocity_x, velocity_y);

        (self.direction, position, velocity)
    }
}