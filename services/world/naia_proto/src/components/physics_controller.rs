use bevy_ecs::component::Component;

use naia_bevy_shared::Tick;
use logging::{info, warn};
use math::Vec2;

use crate::{
    components::{NextTilePosition, velocity::Velocity},
    constants::{
        MOVEMENT_ACCELERATION, MOVEMENT_DECELERATION, MOVEMENT_VELOCITY_MAX, MOVEMENT_VELOCITY_MIN,
        TILE_SIZE,
    },
};

#[derive(Component)]
pub struct PhysicsController {
    position: Vec2,
    velocity: Velocity,
}

impl PhysicsController {
    pub fn new(ntp: &NextTilePosition) -> Self {
        let position = Vec2::new(ntp.x() as f32 * TILE_SIZE, ntp.y() as f32 * TILE_SIZE);

        Self {
            position,
            velocity: Velocity::new(0.0, 0.0),
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_tile_position(&mut self, tile_x: i16, tile_y: i16) {
        let new_position = Vec2::new(tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE);
        self.position = new_position;
    }

    pub fn tick_log(&self, tick: Tick, is_prediction: bool) {
        let prediction = if is_prediction {"PREDICTED"} else {"CONFIRMED"};
        info!("{:?} - tick: {:?}, position: {:?}, velocity: {:?}", prediction, tick, self.position, self.velocity);
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity.get_vec2()
    }

    pub fn set_velocity(&mut self, x: f32, y: f32) {
        self.velocity.set_vec2(Vec2::new(x, y));
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

