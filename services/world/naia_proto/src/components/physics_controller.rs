use bevy_ecs::component::Component;

use logging::warn;
use math::Vec2;
use naia_bevy_shared::Tick;

use crate::{
    components::{velocity::Velocity, NetworkedTileTarget},
    constants::{
        MOVEMENT_ACCELERATION, MOVEMENT_DECELERATION, MOVEMENT_VELOCITY_MAX, MOVEMENT_VELOCITY_MIN,
        TILE_SIZE,
    },
};

#[derive(Component, Clone)]
pub struct PhysicsController {
    position: Vec2,
    velocity: Velocity,
}

impl PhysicsController {
    pub fn new(ntp: &NetworkedTileTarget) -> Self {
        let position = Vec2::new(ntp.x() as f32 * TILE_SIZE, ntp.y() as f32 * TILE_SIZE);

        Self {
            position,
            velocity: Velocity::new(0.0, 0.0),
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, x: f32, y: f32, check_diff: bool) {
        let new_position = Vec2::new(x, y);

        if check_diff {
            let distance = self.position.distance(new_position);
            if distance > 0.0 {
                warn!(
                    "set_position({:?}, {:?}): ({:?}, {:?}) -> ({:?}, {:?}), distance: {:?}",
                    x,
                    y,
                    self.position.x,
                    self.position.y,
                    new_position.x,
                    new_position.y,
                    distance,
                );
            }
        }

        self.position = new_position;
    }

    pub fn set_tile_position(&mut self, tile_x: i16, tile_y: i16, check_diff: bool) {
        let new_position = Vec2::new(tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE);

        if check_diff {
            let distance = self.position.distance(new_position);
            if distance > 0.0 {
                warn!(
                    "set_tile_position({:?}, {:?}): ({:?}, {:?}) -> ({:?}, {:?}), distance: {:?}",
                    tile_x,
                    tile_y,
                    self.position.x,
                    self.position.y,
                    new_position.x,
                    new_position.y,
                    distance,
                );
            }
        }

        self.position = new_position;
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity.get_vec2()
    }

    pub fn set_velocity(&mut self, x: f32, y: f32, check_diff: bool) {
        let old_velocity = self.velocity.get_vec2();
        let new_velocity = Vec2::new(x, y);

        if check_diff {
            let distance = old_velocity.distance(new_velocity);
            if distance > 0.0 {
                warn!(
                    "set_velocity(): ({:?}, {:?}) -> ({:?}, {:?}), distance: {:?}",
                    old_velocity.x, old_velocity.y, new_velocity.x, new_velocity.y, distance,
                );
            }
        }

        self.velocity.set_vec2(new_velocity);
    }

    pub fn tick_log(&self, _tick: Tick, _is_prediction: bool) {
        // let prediction = if is_prediction {"PREDICTED"} else {"CONFIRMED"};
        // let velocity = self.velocity.get_vec2();
        // info!(
        //     "{:?} - tick: {:?}, position: ({:?}, {:?}), velocity: ({:?}, {:?})",
        //     prediction, tick, self.position.x, self.position.y, velocity.x, velocity.y
        // );
    }

    pub fn speed_up(&mut self, target_direction: Vec2) {
        // let old_velocity = self.velocity;
        let velocity_vec2 = self.velocity.get_vec2();
        let length = velocity_vec2.length();

        let current_normal = velocity_vec2.normalize_or_zero();
        let target_normal = target_direction.normalize_or_zero();
        let new_normal = Vec2::new(
            (current_normal.x + target_normal.x) / 2.0,
            (current_normal.y + target_normal.y) / 2.0,
        );
        let new_length = (length + MOVEMENT_ACCELERATION).min(MOVEMENT_VELOCITY_MAX);

        self.velocity.set_vec2(new_normal * new_length);

        // info!("speed_up() .. old velocity: {:?}, new velocity: {:?}", old_velocity, self.velocity);
    }

    pub fn slow_down(&mut self, target_direction: Vec2) {
        // let old_velocity = self.velocity;
        let velocity_vec2 = self.velocity.get_vec2();
        let length = velocity_vec2.length();
        let current_normal = velocity_vec2.normalize_or_zero();
        let target_normal = target_direction.normalize_or_zero();
        let new_normal = Vec2::new(
            (current_normal.x + target_normal.x) / 2.0,
            (current_normal.y + target_normal.y) / 2.0,
        );
        let new_length = (length - MOVEMENT_DECELERATION).max(MOVEMENT_VELOCITY_MIN);

        self.velocity.set_vec2(new_normal * new_length);

        // info!("slow_down() .. old velocity: {:?}, new velocity: {:?}", old_velocity, self.velocity);
    }

    pub fn step(&mut self) {
        self.position += self.velocity.get_vec2();
    }

    pub fn recv_rollback(&mut self, tick: Tick, other: &Self) {
        self.position = other.position;
        self.velocity.set_vec2(other.velocity.get_vec2());

        self.tick_log(tick, true);
    }
}
