use bevy_ecs::component::Component;

use math::Vec2;

use crate::{
    components::NextTilePosition,
    constants::{
        MOVEMENT_ACCELERATION, MOVEMENT_DECELERATION, MOVEMENT_VELOCITY_MAX, MOVEMENT_VELOCITY_MIN,
        TILE_SIZE,
    },
};

#[derive(Component)]
pub struct PhysicsController {
    position: Vec2,
    velocity: Vec2,
}

impl PhysicsController {
    pub fn new(ntp: &NextTilePosition) -> Self {
        let position = Vec2::new(ntp.x() as f32 * TILE_SIZE, ntp.y() as f32 * TILE_SIZE);

        Self {
            position,
            velocity: Vec2::new(0.0, 0.0),
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn speed_up(&mut self, target_direction: Vec2) {
        // let old_velocity = self.velocity;
        let length = self.velocity.length();

        let current_normal = self.velocity.normalize_or_zero();
        let target_normal = target_direction.normalize_or_zero();
        let new_normal = Vec2::new(
            (current_normal.x + target_normal.x) / 2.0,
            (current_normal.y + target_normal.y) / 2.0,
        );
        let new_length = (length + MOVEMENT_ACCELERATION).min(MOVEMENT_VELOCITY_MAX);

        self.velocity = new_normal * new_length;

        // info!("speed_up() .. old velocity: {:?}, new velocity: {:?}", old_velocity, self.velocity);
    }

    pub fn slow_down(&mut self, target_direction: Vec2) {
        // let old_velocity = self.velocity;
        let length = self.velocity.length();
        let current_normal = self.velocity.normalize_or_zero();
        let target_normal = target_direction.normalize_or_zero();
        let new_normal = Vec2::new(
            (current_normal.x + target_normal.x) / 2.0,
            (current_normal.y + target_normal.y) / 2.0,
        );
        let new_length = (length - MOVEMENT_DECELERATION).max(MOVEMENT_VELOCITY_MIN);

        self.velocity = new_normal * new_length;

        // info!("slow_down() .. old velocity: {:?}, new velocity: {:?}", old_velocity, self.velocity);
    }

    pub fn step(&mut self) {
        self.position += self.velocity;
    }

    pub fn recv_rollback(&mut self, other: &Self) {
        self.position = other.position;
        self.velocity = other.velocity;
    }
}
