use bevy_ecs::component::Component;

use logging::warn;
use math::Vec2;
use naia_bevy_shared::Tick;

use crate::{
    components::{velocity::Velocity, NetworkedTileTarget},
    constants::{
        MOVEMENT_ACCELERATION, MOVEMENT_VELOCITY_MAX, MOVEMENT_ARRIVAL_DISTANCE,
        TILE_SIZE, MOVEMENT_FRICTION, MOVEMENT_STEERING_DEADZONE
    },
    types::Direction,
};
use crate::constants::{MOVEMENT_STOPPING_DISTANCE, MOVEMENT_VELOCITY_MIN};

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

    pub fn update_velocity(
        &mut self,
        current_direction: Direction,
        target_position: Vec2,
        next_direction: Option<Direction>
    ) {

        let current_velocity = self.velocity.get_vec2();
        let current_position = self.position;

        let new_velocity = update(
            current_position,
            current_velocity,
            current_direction,
            target_position,
            next_direction,
        );

        self.velocity.set_vec2(new_velocity);
    }

    pub fn step(&mut self) {
        self.position += self.velocity.get_vec2();
    }
}

fn update(
    current_position: Vec2,
    current_velocity: Vec2,
    current_direction: Direction,
    target_position: Vec2,
    future_direction: Option<Direction>
) -> Vec2 {
    let mut output_velocity = current_velocity;

    let steering = find_steering(current_position, current_velocity, current_direction, target_position, future_direction);
    let control_signal = steering_to_control_signal(steering);
    apply_locomotion(control_signal, &mut output_velocity);
    apply_limitations(&mut output_velocity);

    output_velocity
}

fn find_steering(
    current_position: Vec2,
    current_velocity: Vec2,
    current_direction: Direction,
    target_position: Vec2,
    future_direction: Option<Direction>
) -> Vec2 {
    if let Some(future_direction) = future_direction {
        // ARRIVE WITH VELOCITY BEHAVIOR
        let current_target_position = target_position;
        let future_target_position = {
            let (dx, dy) = future_direction.to_delta();
            let offset = Vec2::new(dx as f32, dy as f32) * TILE_SIZE;
            current_target_position + offset
        };
        let starting_position = {
            let (dx, dy) = current_direction.to_delta();
            let offset = Vec2::new(dx as f32, dy as f32) * TILE_SIZE;
            current_target_position - offset
        };
        let current_target_dir = (current_target_position - starting_position).normalize_or_zero();
        let future_target_dir = (future_target_position - current_target_position).normalize_or_zero();
        let blended_dir = (current_target_dir + future_target_dir).normalize_or_zero();
        if blended_dir.length() < MOVEMENT_FRICTION {
            return find_steering_arrival(current_position, current_velocity, current_target_position);
        }
        let target_velocity = blended_dir * MOVEMENT_VELOCITY_MAX;

        // Find steering such that the entity passes through current_target_position with the given target_velocity

        // Find an auxiliary position that would allow entity to reach current_target_position with target_velocity
        let aux_target_position = closest_point_on_a_ray(
            current_target_position,
            -target_velocity,
            current_position,
        ).unwrap_or(current_target_position);
        let aux_target_offset = aux_target_position - current_position;
        let aux_target_distance = aux_target_offset.length();
        let aux_desired_velocity = aux_target_offset.normalize_or_zero() * MOVEMENT_VELOCITY_MAX;

        // Find target desired velocity
        let current_target_offset = current_target_position - current_position;
        let current_target_distance = current_target_offset.length();
        let target_desired_velocity = current_target_offset.normalize_or_zero() * MOVEMENT_VELOCITY_MAX;

        // Blend aux_desired_velocity with target_desired_velocity
        let desired_velocity = {
            let total_distance = current_target_distance + aux_target_distance;
            const AUX_PREFERENCE: f32 = 0.5; // 0.0 = target, 1.0 = aux
            let aux_weight = ((aux_target_distance / total_distance) * AUX_PREFERENCE) + (1.0 - AUX_PREFERENCE);
            let target_weight = 1.0 - aux_weight;
            (aux_desired_velocity * aux_weight) + (target_desired_velocity * target_weight)
        };

        return desired_velocity - current_velocity;
    } else {
        return find_steering_arrival(current_position, current_velocity, target_position);
    }
}

fn find_steering_arrival(current_position: Vec2, current_velocity: Vec2, target_position: Vec2) -> Vec2 {
    // ARRIVAL BEHAVIOR
    let target_offset = target_position - current_position;
    let target_distance = target_offset.length();
    let target_direction = target_offset.normalize_or_zero();

    let accelerate_to_max_speed = {
        let current_speed = current_velocity.length();
        if current_speed <= MOVEMENT_VELOCITY_MIN {
            true
        } else {
            let ticks_to_target = target_distance / current_speed;
            let tick_to_deacc = (current_speed - MOVEMENT_VELOCITY_MIN) / MOVEMENT_ACCELERATION;
            if ticks_to_target > tick_to_deacc {
                true
            } else {
                false
            }
        }
    };

    let desired_velocity = if accelerate_to_max_speed {
        // SEEK / PASS-THROUGH
        target_direction * MOVEMENT_VELOCITY_MAX
    } else {
        // ARRIVAL
        target_direction * MOVEMENT_VELOCITY_MIN
    };

    desired_velocity - current_velocity
}

fn steering_to_control_signal(steering: Vec2) -> Option<Direction> {
    if steering.length() < MOVEMENT_STEERING_DEADZONE {
        return None;
    }

    let direction = Direction::from_coords(steering.x, steering.y);
    return Some(direction);
}

fn apply_locomotion(control_signal: Option<Direction>, velocity: &mut Vec2) {
    if let Some(control_signal) = control_signal {
        // control signal exists, apply acceleration
        let (dx, dy) = control_signal.to_delta();
        let mut acceleration = Vec2::new(dx as f32, dy as f32).normalize_or_zero();
        acceleration = acceleration * MOVEMENT_ACCELERATION;

        *velocity += acceleration;
    } else {
        // no control signal, allow friction to slow down
        if velocity.length() > MOVEMENT_FRICTION {
            // apply friction
            let friction = velocity.normalize_or_zero() * MOVEMENT_FRICTION;
            *velocity -= friction;
        } else {
            // friction completely kills velocity
            *velocity = Vec2::ZERO;
        }
    }
}

fn apply_limitations(velocity: &mut Vec2) {

    // limit max speed
    if velocity.length() > MOVEMENT_VELOCITY_MAX {
        *velocity = velocity.normalize_or_zero() * MOVEMENT_VELOCITY_MAX;
    }
}

// a is the start point of a ray, ab is the direction of the ray, c is the target point
fn closest_point_on_a_ray(
    ray_start: Vec2,
    ray_dir: Vec2,
    target: Vec2
) -> Option<Vec2> {

    // If (a == b), the line is degenerate; just return a.
    if ray_dir.length_squared() < f32::EPSILON {
        return Some(ray_start); // TODO: should be None here?
    }

    // Vector from a to c
    let ac = target - ray_start;

    // Project ac onto ab to find parameter t
    let t = ac.dot(ray_dir) / ray_dir.dot(ray_dir);

    if t < 0.0 {
        return None;
    }

    // The closest point on the line is then a + t * ab
    return Some(ray_start + ray_dir * t)
}