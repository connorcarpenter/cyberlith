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

    pub fn get_steering_vars(
        &self,
        current_direction: Direction,
        current_target_position: Vec2,
        next_direction: Option<Direction>
    ) -> Option<(Vec2, Vec2)> {
        let current_position = self.position();

        if next_direction.is_none() {
            if current_position.distance(current_target_position) <= MOVEMENT_ARRIVAL_DISTANCE {
                // arrived!
                return None;
            }
        }

        let axis_ray = {
            let (dx, dy) = current_direction.to_delta();
            Vec2::new(dx as f32 * -1.0, dy as f32 * -1.0).normalize_or_zero()
        };

        let Some(axis_ray_nearest_point) = closest_point_on_a_ray(
            current_target_position,
            axis_ray,
            current_position,
        ) else {
            // arrived!
            return None;
        };

        return Some((axis_ray, axis_ray_nearest_point));
    }

    pub fn update_velocity(
        &mut self,
        current_direction: Direction,
        current_target_position: Vec2,
        future_direction: Option<Direction>,
        axis_ray: Vec2,
        axis_ray_nearest_point: Vec2,
    ) {
        let current_position = self.position;
        let current_velocity = self.velocity.get_vec2();

        let new_velocity = update(
            current_position,
            current_velocity,
            current_direction,
            current_target_position,
            future_direction,
            axis_ray,
            axis_ray_nearest_point,
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
    current_target_position: Vec2,
    future_direction: Option<Direction>,
    axis_ray: Vec2,
    axis_ray_nearest_point: Vec2,
) -> Vec2 {
    let mut output_velocity = current_velocity;

    let control_signal = find_steering(
        current_position,
        current_velocity,
        current_direction,
        current_target_position,
        future_direction,
        axis_ray,
        axis_ray_nearest_point,
    );
    apply_locomotion(control_signal, &mut output_velocity);
    apply_limitations(&mut output_velocity);

    output_velocity
}

fn find_steering(
    current_position: Vec2,
    current_velocity: Vec2,
    current_direction: Direction,
    target_position: Vec2,
    future_direction: Option<Direction>,
    axis_ray: Vec2,
    axis_ray_nearest_point: Vec2,
) -> Option<Direction> {

    let axis_distance_to_target = axis_ray_nearest_point.distance(target_position);
    if future_direction.is_none() {
        // real_distance_to_target IS > MOVEMENT_ARRIVAL_DISTANCE, otherwise wouldn't be here
        if axis_distance_to_target <= MOVEMENT_ARRIVAL_DISTANCE {
            // we have overshot
            let target_offset = target_position - current_position;
            let target_distance = target_offset.length();
            let target_direction = target_offset.normalize_or_zero();
            let desired_velocity = target_direction * MOVEMENT_VELOCITY_MIN;
            let desired_acceleration = desired_velocity - current_velocity;
            return Some(Direction::from_coords(desired_acceleration.x, desired_acceleration.y));
        }
    }

    let real_distance_to_target = current_position.distance(target_position);
    let offset_to_axis = axis_ray_nearest_point - current_position;
    let distance_to_axis = offset_to_axis.length();
    let direction_to_axis = offset_to_axis.normalize_or_zero();

    let forward_direction = axis_ray * -1.0;
    let left_direction = Vec2::new(-forward_direction.y, forward_direction.x);
    let forward_speed = current_velocity.dot(forward_direction);
    let left_speed = current_velocity.dot(left_direction);
    let left_velocity = left_direction * left_speed;
    let left_speed_to_axis = left_velocity.dot(direction_to_axis);

    // forward_control_x & _y are -1, 0, or 1 here
    let (mut forward_control_x, mut forward_control_y) = current_direction.to_delta();
    if forward_control_x == 0 || forward_control_y == 0 {
        // forward is orthogonal
        if forward_control_x == 0 {
            // forward is on vertical axis

            // steer to axis
            {
                let accelerate_to_axis = {

                    if left_speed_to_axis < 0.0 {
                        // currently moving away from axis
                        true
                    } else {
                        // currently moving towards axis
                        let left_speed_abs = left_speed.abs();
                        let ticks_to_target = distance_to_axis / left_speed_abs;
                        let tick_to_deacc = left_speed_abs / MOVEMENT_FRICTION;
                        if ticks_to_target > tick_to_deacc {
                            true
                        } else {
                            false
                        }
                    }
                };

                if accelerate_to_axis {
                    if offset_to_axis.x > 0.0 {
                        forward_control_x = 1;
                    } else {
                        forward_control_x = -1;
                    }
                }
            }

            // steer to target
            {
                // the question here is, do we set forward_control_y to 0 or not?
                let accelerate_to_target = {

                    if forward_speed < 0.0 {
                        // currently moving away from target
                        true
                    } else {
                        // currently moving towards target
                        let forward_speed_abs = forward_speed.abs();
                        let ticks_to_target = axis_distance_to_target / forward_speed_abs;
                        let tick_to_deacc = forward_speed_abs / MOVEMENT_FRICTION;
                        if ticks_to_target > tick_to_deacc {
                            true
                        } else {
                            false
                        }
                    }
                };

                if !accelerate_to_target {
                    forward_control_y = 0;
                }
            }

            Direction::from_delta(forward_control_x, forward_control_y)
        } else {
            // forward is on horizontal axis

            // steer to axis
            {
                let accelerate_to_axis = {

                    if left_speed_to_axis < 0.0 {
                        // currently moving away from axis
                        true
                    } else {
                        // currently moving towards axis
                        let left_speed_abs = left_speed.abs();
                        let ticks_to_target = distance_to_axis / left_speed_abs;
                        let tick_to_deacc = left_speed_abs / MOVEMENT_FRICTION;
                        if ticks_to_target > tick_to_deacc {
                            true
                        } else {
                            false
                        }
                    }
                };

                if accelerate_to_axis {
                    if offset_to_axis.y > 0.0 {
                        forward_control_y = 1;
                    } else {
                        forward_control_y = -1;
                    }
                }
            }

            // steer to target
            {
                // the question here is, do we set forward_control_x to 0 or not?
                let accelerate_to_target = {

                    if forward_speed < 0.0 {
                        // currently moving away from target
                        true
                    } else {
                        // currently moving towards target
                        let forward_speed_abs = forward_speed.abs();
                        let ticks_to_target = axis_distance_to_target / forward_speed_abs;
                        let tick_to_deacc = forward_speed_abs / MOVEMENT_FRICTION;
                        if ticks_to_target > tick_to_deacc {
                            true
                        } else {
                            false
                        }
                    }
                };

                if !accelerate_to_target {
                    forward_control_x = 0;
                }
            }

            Direction::from_delta(forward_control_x, forward_control_y)
        }
    } else {
        // forward is diagonal
        todo!();

        Direction::from_delta(forward_control_x, forward_control_y)
    }
}

fn apply_locomotion(
    control_signal: Option<Direction>,
    velocity: &mut Vec2,
) {
    if let Some(control_signal) = control_signal {
        // control signal exists, apply acceleration
        let (dx, dy) = control_signal.to_delta();
        let acceleration = Vec2::new(dx as f32, dy as f32).normalize_or_zero()  * MOVEMENT_ACCELERATION;

        *velocity += acceleration;

        // apply friction against backwards velocity
        let forward_direction = acceleration.normalize_or_zero();
        let forward_speed = velocity.dot(forward_direction);
        if forward_speed < 0.0 {
            // currently moving backwards, apply friction
            let friction = forward_direction * MOVEMENT_FRICTION;
            *velocity += friction;
        }

        // apply friction against sideways velocity
        let left_direction = Vec2::new(-forward_direction.y, forward_direction.x);
        let left_speed = velocity.dot(left_direction);
        let friction = if left_speed < 0.0 {
            // currently moving left
            left_direction * MOVEMENT_FRICTION

        } else {
            // currently moving right
            left_direction * MOVEMENT_FRICTION * -1.0
        };
        *velocity += friction;
    } else {
        // no control signal, apply friction
        if velocity.length() > MOVEMENT_FRICTION {
            let friction = velocity.normalize_or_zero() * MOVEMENT_FRICTION * -1.0;
            *velocity += friction;
        } else {
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

fn side_of_line(
    line_start: Vec2,
    line_dir: Vec2,
    target: Vec2
) -> i8 {
    // Compute the vector from the line anchor to the target
    let pt_to_target = target - line_start;

    // 2D cross product of line_dir and pt_to_target
    let cross_val = line_dir.x * pt_to_target.y - line_dir.y * pt_to_target.x;

    if cross_val.abs() < 1e-7 {
        0  // ~ collinear
    } else if cross_val > 0.0 {
        1  // "left"
    } else {
        -1 // "right"
    }
}