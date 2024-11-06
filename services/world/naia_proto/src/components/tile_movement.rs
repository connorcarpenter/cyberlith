
use bevy_ecs::prelude::Component;

use naia_bevy_shared::Tick;

use logging::{warn};
use math::Vec2;

use crate::{
    components::NextTilePosition,
    constants::{MOVEMENT_VELOCITY_MAX, TILE_SIZE},
    messages::PlayerCommands,
    types::Direction,
};
use crate::constants::{MOVEMENT_ACCELERATION, MOVEMENT_DECELERATION};

#[derive(Component)]
pub struct TileMovement {
    is_server: bool,
    is_predicted: bool,

    state: TileMovementState,
}

impl TileMovement {
    pub fn new_stopped(
        is_server: bool,
        predicted: bool,
        next_tile_position: &NextTilePosition,
    ) -> Self {
        if is_server && predicted {
            panic!("Server entities cannot be predicted");
        }
        let me = Self {
            is_server,
            is_predicted: predicted,

            state: TileMovementState::stopped(next_tile_position.x(), next_tile_position.y()),
        };

        me
    }

    // retrieve the current position of the entity
    pub fn current_position(&self) -> Vec2 {
        match &self.state {
            TileMovementState::Stopped(state) => state.current_position(),
            TileMovementState::Moving(state) => state.current_position(),
        }
    }

    // retrieve the current tile position of the entity
    pub fn current_tile_position(&self) -> (i16, i16) {
        match &self.state {
            TileMovementState::Stopped(state) => state.current_tile_position(),
            TileMovementState::Moving(_state) => panic!("Expected Stopped state"),
        }
    }

    pub fn to_tile_position(&self) -> (i16, i16) {
        match &self.state {
            TileMovementState::Stopped(_state) => panic!("Expected Moving state"),
            TileMovementState::Moving(state) => (state.to_tile_x, state.to_tile_y),
        }
    }

    // return whether the entity is moving
    pub fn is_moving(&self) -> bool {
        self.state.is_moving()
    }

    // return whether the entity is stopped
    pub fn is_stopped(&self) -> bool {
        self.state.is_stopped()
    }

    // return whether the entity is moving but done
    pub fn is_done(&self) -> bool {
        self.state.is_done()
    }

    pub fn set_stopped(&mut self, tile_x: i16, tile_y: i16) {
        if !self.is_moving() {
            panic!("Cannot set stopped state when not moving");
        }
        self.state = TileMovementState::stopped(tile_x, tile_y);
    }

    pub fn set_moving(&mut self, move_dir: Direction) {
        if !self.is_stopped() {
            panic!("Cannot set moving state when not stopped");
        }
        let (current_tile_x, current_tile_y) = self.current_tile_position();
        self.state = TileMovementState::moving(current_tile_x, current_tile_y, move_dir);
    }

    pub fn set_continue(&mut self, tile_x: i16, tile_y: i16, move_dir: Direction) {
        if !self.is_moving() {
            panic!("Cannot set continue state when not moving");
        }

        if !self.is_done() {
            panic!("Expected done state");
        }

        let position = self.current_position();
        let velocity = self.state.current_velocity();

        self.state = TileMovementState::continuing(tile_x, tile_y, move_dir, position, velocity);
    }

    // call on each tick
    pub fn process_tick(
        &mut self,
        tick: Tick,
        player_command: Option<PlayerCommands>,
    ) -> (ProcessTickResult, Option<(i16, i16)>) {
        let mut output = None;

        if self.is_predicted || self.is_server {
            output = self.process_command(tick, player_command);
        }

        let mut result = match &mut self.state {
            TileMovementState::Stopped(state) => state.process_tick(),
            TileMovementState::Moving(state) => state.process_tick(),
        };

        if let ProcessTickResult::ShouldContinue(tile_x, tile_y, buffered_move_dir) = result {
            self.set_continue(tile_x, tile_y, buffered_move_dir);

            if self.is_server {
                let (dx, dy) = buffered_move_dir.to_delta();

                let next_tile_x = tile_x + dx as i16;
                let next_tile_y = tile_y + dy as i16;

                output = Some((next_tile_x, next_tile_y));
            }

            result = ProcessTickResult::DoNothing;

        }

        return (result, output);
    }

    // on the client, called by predicted entities
    // on the server, called by confirmed entities
    fn process_command(&mut self, tick: Tick, command: Option<PlayerCommands>) -> Option<(i16, i16)> {
        if !self.is_server && !self.is_predicted {
            panic!("Only predicted entities can receive commands");
        }

        let Some(command) = command else {
            return None;
        };
        let Some(direction) = command.get_move() else {
            return None;
        };

        if self.state.is_moving() {

            if self.state.can_buffer_movement() {
                self.state.buffer_movement(tick, direction);
            }

        } else {

            let TileMovementState::Stopped(state) = &self.state else {
                panic!("Expected Stopped state");
            };

            let (dx, dy) = direction.to_delta();

            let next_tile_x = state.tile_x + dx as i16;
            let next_tile_y = state.tile_y + dy as i16;

            self.set_moving(direction);

            if self.is_server {
                return Some((next_tile_x, next_tile_y));
            }
        }

        return None;
    }

    pub fn mirror(&mut self, other: &TileMovement) {
        self.state = other.state.clone();
    }
}

// Tile Movement State
#[derive(Clone)]
enum TileMovementState {
    Stopped(TileMovementStoppedState),
    Moving(TileMovementMovingState),
}

impl TileMovementState {
    fn stopped(tile_x: i16, tile_y: i16) -> Self {
        Self::Stopped(TileMovementStoppedState::new(tile_x, tile_y))
    }

    fn moving(tile_x: i16, tile_y: i16, move_dir: Direction) -> Self {
        Self::Moving(TileMovementMovingState::new(tile_x, tile_y, move_dir, None, None))
    }

    fn continuing(tile_x: i16, tile_y: i16, move_dir: Direction, position: Vec2, velocity: Vec2) -> Self {
        Self::Moving(TileMovementMovingState::new(tile_x, tile_y, move_dir, Some(position), Some(velocity)))
    }

    fn is_stopped(&self) -> bool {
        match self {
            Self::Stopped(_) => true,
            Self::Moving(_) => false,
        }
    }

    fn is_moving(&self) -> bool {
        match self {
            Self::Stopped(_) => false,
            Self::Moving(_) => true,
        }
    }

    fn is_done(&self) -> bool {
        match self {
            Self::Stopped(state) => panic!("Expected Moving state"),
            Self::Moving(state) => state.done,
        }
    }

    fn can_buffer_movement(&self) -> bool {
        match self {
            Self::Stopped(_) => false,
            Self::Moving(state) => state.can_buffer_movement(),
        }
    }

    pub(crate) fn buffer_movement(&mut self, tick: Tick, move_dir: Direction) {
        match self {
            Self::Stopped(_) => {},
            Self::Moving(state) => state.buffer_movement(tick, move_dir),
        }
    }

    pub fn current_velocity(&self) -> Vec2 {
        match self {
            Self::Stopped(_) => panic!("Expected Moving state"),
            Self::Moving(state) => state.current_velocity(),
        }
    }
}

// Tile Movement Stopped State
#[derive(Clone)]
struct TileMovementStoppedState {
    tile_x: i16,
    tile_y: i16,
}

impl TileMovementStoppedState {
    fn new(tile_x: i16, tile_y: i16) -> Self {
        Self { tile_x, tile_y }
    }

    // retrieve the current position of the entity
    fn current_position(&self) -> Vec2 {
        Vec2::new(
            self.tile_x as f32 * TILE_SIZE,
            self.tile_y as f32 * TILE_SIZE,
        )
    }

    // retrieve the current tile position of the entity
    fn current_tile_position(&self) -> (i16, i16) {
        (
            self.tile_x,
            self.tile_y,
        )
    }

    // call on each tick
    fn process_tick(&mut self) -> ProcessTickResult {
        return ProcessTickResult::DoNothing;
    }
}

// Tile Movement Moving State
#[derive(Clone)]
struct TileMovementMovingState {
    from_tile_x: i16,
    from_tile_y: i16,
    to_tile_x: i16,
    to_tile_y: i16,
    done: bool,

    position: Vec2,
    velocity: Vec2,

    buffered_move_dir: Option<Direction>,
}

impl TileMovementMovingState {

    fn new(from_tile_x: i16, from_tile_y: i16, move_dir: Direction, position_opt: Option<Vec2>, velocity_opt: Option<Vec2>) -> Self {

        let (dx, dy) = move_dir.to_delta();
        let to_tile_x = from_tile_x + dx as i16;
        let to_tile_y = from_tile_y + dy as i16;

        let position = position_opt.unwrap_or_else(|| {
            Vec2::new(
                from_tile_x as f32 * TILE_SIZE,
                from_tile_y as f32 * TILE_SIZE,
            )
        });
        let velocity = velocity_opt.unwrap_or_else(|| {
            Vec2::ZERO
        });

        Self {
            from_tile_x,
            from_tile_y,
            to_tile_x,
            to_tile_y,
            position,
            velocity,
            done: false,
            buffered_move_dir: None,
        }
    }

    // retrieve the current position of the entity
    pub(crate) fn current_position(&self) -> Vec2 {

        return self.position;



        // let interp = self.distance / self.distance_max; // this is what is varying across frames
        //
        // let from_x = self.from_tile_x as f32 * TILE_SIZE;
        // let from_y = self.from_tile_y as f32 * TILE_SIZE;
        // let to_x = self.to_tile_x as f32 * TILE_SIZE;
        // let to_y = self.to_tile_y as f32 * TILE_SIZE;
        //
        // let dis_x = to_x - from_x;
        // let dis_y = to_y - from_y;
        //
        // (from_x + (dis_x * interp), from_y + (dis_y * interp))
    }

    pub(crate) fn current_velocity(&self) -> Vec2 {
        return self.velocity;
    }

    pub(crate) fn target_position(&self) -> Vec2 {
        return Vec2::new(
            self.to_tile_x as f32 * TILE_SIZE,
            self.to_tile_y as f32 * TILE_SIZE,
        );
    }

    pub(crate) fn distance_to_target(&self) -> f32 {
        let target_position = self.target_position();
        let distance = self.position.distance(target_position);
        return distance;
    }

    // call on each tick
    fn process_tick(&mut self) -> ProcessTickResult {
        if self.done {
            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
        }

        let target_distance = self.distance_to_target();
        const STOPPING_DISTANCE: f32 = 0.5 * TILE_SIZE;
        let target_position = self.target_position();
        let target_direction = (target_position - self.position).normalize();

        if target_distance > STOPPING_DISTANCE {
            // speed up
            self.velocity += target_direction * MOVEMENT_ACCELERATION;
            if self.velocity.length() > MOVEMENT_VELOCITY_MAX {
                self.velocity = self.velocity.normalize() * MOVEMENT_VELOCITY_MAX;
            }
        } else {
            // slow down
            if self.velocity.length() > MOVEMENT_DECELERATION {
                self.velocity -= target_direction * MOVEMENT_DECELERATION;
            }
        }

        if target_distance <= self.velocity.length() {

            // reached target!

            self.done = true;

            if self.buffered_move_dir.is_none() {
                return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
            } else {
                let buffered_move_dir = self.buffered_move_dir.take().unwrap();
                return ProcessTickResult::ShouldContinue(self.to_tile_x, self.to_tile_y, buffered_move_dir);
            }
        } else {

            self.position += self.velocity;

            return ProcessTickResult::DoNothing;
        }
    }

    pub(crate) fn can_buffer_movement(&self) -> bool {
        // return false;
        return self.distance_to_target() < TILE_SIZE * 0.5; // TODO: this should be better
    }

    pub(crate) fn buffer_movement(&mut self, tick: Tick, move_dir: Direction) {
        warn!("buffering command for Tick: {:?}, MoveDir: {:?}", tick, move_dir);

        self.buffered_move_dir = Some(move_dir);
    }
}

pub enum ProcessTickResult {
    ShouldStop(i16, i16),
    ShouldContinue(i16, i16, Direction),
    DoNothing,
}

fn get_tile_distance(ax: i16, ay: i16, bx: i16, by: i16) -> f32 {

    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    let d_dis = dx + dy;

    if d_dis == 0 || d_dis > 2 || dx > 1 || dy > 1 {
        panic!("invalid distance between tiles!");
    }

    let x_axis_changed: bool = ax != bx;
    let y_axis_changed: bool = ay != by;

    let distance_max = if x_axis_changed && y_axis_changed {
        std::f32::consts::SQRT_2
    } else {
        1.0
    };
    let distance_max = distance_max * TILE_SIZE;
    distance_max
}