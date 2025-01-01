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
use crate::constants::{MOVEMENT_FRICTION, MOVEMENT_NUDGE_MAX};
use crate::types::Direction;

#[derive(Component, Clone)]
pub struct PhysicsController {
    position: Vec2,
    velocity: Velocity,
    nudge: Option<Vec2>,
}

impl PhysicsController {
    pub fn new(ntp: &NetworkedTileTarget) -> Self {
        let position = Vec2::new(ntp.x() as f32 * TILE_SIZE, ntp.y() as f32 * TILE_SIZE);

        Self {
            position,
            velocity: Velocity::new(0.0, 0.0),
            nudge: None,
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

    pub fn accelerate(&mut self, dir: Direction, target_position: Vec2, has_future: bool) {

        let mut velocity = self.velocity.get_vec2();

        // friction
        let speed = velocity.length();
        if speed > MOVEMENT_FRICTION {
            velocity = velocity.normalize_or_zero() * (speed - MOVEMENT_FRICTION);
        } else {
            velocity = Vec2::ZERO;
        }

        let mut accelerate_to_max_speed = has_future;
        if !accelerate_to_max_speed {
            let speed = velocity.length();
            if speed <= MOVEMENT_VELOCITY_MIN {
                accelerate_to_max_speed = true;
            } else {
                let target_distance = self.position.distance(target_position);
                let ticks_to_target = target_distance / speed;
                let tick_to_deacc = (speed - MOVEMENT_VELOCITY_MIN) / MOVEMENT_DECELERATION;
                if ticks_to_target > tick_to_deacc {
                    accelerate_to_max_speed = true;
                } else {
                    accelerate_to_max_speed = false;
                }
            }
        }

        // find acceleration
        let (tdx, tdy) = dir.to_delta();
        let target_direction = Vec2::new(tdx as f32, tdy as f32);
        let target_direction = target_direction.normalize_or_zero();

        let target_velocity = if accelerate_to_max_speed {
            // SEEK / PASS-THROUGH
            target_direction * MOVEMENT_VELOCITY_MAX
        } else {
            // ARRIVAL
            target_direction * MOVEMENT_VELOCITY_MIN
        };
        let mut acceleration = target_velocity - velocity;

        // nudge
        {
            if let Some(closest_point) = closest_point_on_a_line(self.position, target_velocity, target_position) {
                let mut target_diff = target_position - closest_point;
                let target_length = target_diff.length();
                if target_length > 0.0 {
                    if target_length > MOVEMENT_NUDGE_MAX {
                        target_diff = target_diff.normalize_or_zero() * MOVEMENT_NUDGE_MAX;
                    }
                    self.nudge = Some(target_diff);
                } else {
                    self.nudge = None;
                }
            } else {
                // we have overshot the target!
                let target_velocity = (target_position - self.position).normalize_or_zero() * MOVEMENT_VELOCITY_MIN;
                acceleration = target_velocity - velocity;

                self.nudge = None;
            }
        };

        //

        let accelerating = acceleration.dot(velocity) > 0.0;
        let accelerate_max = if accelerating { MOVEMENT_ACCELERATION } else { MOVEMENT_DECELERATION };
        if acceleration.length() > accelerate_max {
            acceleration = acceleration.normalize_or_zero() * accelerate_max;
        }

        velocity += acceleration;

        let speed = velocity.length();
        if speed > MOVEMENT_VELOCITY_MAX {
            velocity = velocity.normalize_or_zero() * MOVEMENT_VELOCITY_MAX;
        }

        self.velocity.set_vec2(velocity);
    }

    pub fn step(&mut self) {
        self.position += self.velocity.get_vec2();

        if let Some(nudge) = self.nudge.take() {
            self.position += nudge;
        }
    }
}

pub fn closest_point_on_a_line(a: Vec2, ab: Vec2, c: Vec2) -> Option<Vec2> {

    // If (a == b), the line is degenerate; just return a.
    if ab.length_squared() < f32::EPSILON {
        return Some(a); // TODO: should be None here?
    }

    // Vector from a to c
    let ac = c - a;

    // Project ac onto ab to find parameter t
    let t = ac.dot(ab) / ab.dot(ab);

    if t < 0.0 {
        return None;
    }

    // The closest point on the line is then a + t * ab
    return Some(a + ab * t)
}