use naia_bevy_shared::Tick;

use logging::{info, warn};
use math::Vec2;

use crate::{
    components::{NextTilePosition, PhysicsController},
    constants::TILE_SIZE,
    messages::PlayerCommands,
    types::Direction,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TileMovementType {
    Server,
    ClientConfirmed,
    ClientPredicted,
}

impl TileMovementType {
    pub fn processes_commands(&self) -> bool {
        match self {
            TileMovementType::Server => true,
            TileMovementType::ClientConfirmed => false,
            TileMovementType::ClientPredicted => true,
        }
    }

    pub fn is_server(&self) -> bool {
        match self {
            TileMovementType::Server => true,
            TileMovementType::ClientConfirmed => false,
            TileMovementType::ClientPredicted => false,
        }
    }
}

#[derive(Clone)]
pub struct TileMovement {
    state: TileMovementState,
}

impl TileMovement {
    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        let me = Self {
            state: TileMovementState::Stopped(TileMovementStoppedState::new(
                next_tile_position.x(),
                next_tile_position.y(),
            )),
        };

        me
    }

    // retrieve the current tile position of the entity
    pub fn tile_position(&self) -> (i16, i16) {
        match &self.state {
            TileMovementState::Stopped(state) => (state.tile_x, state.tile_y),
            TileMovementState::Moving(state) => (state.to_tile_x, state.to_tile_y),
        }
    }

    // return whether the entity is moving
    pub fn is_moving(&self) -> bool {
        match &self.state {
            TileMovementState::Stopped(_) => false,
            TileMovementState::Moving(_) => true,
        }
    }

    // return whether the entity is stopped
    pub fn is_stopped(&self) -> bool {
        match &self.state {
            TileMovementState::Stopped(_) => true,
            TileMovementState::Moving(_) => false,
        }
    }

    // return whether the entity is moving but done
    pub fn is_done(&self) -> bool {
        match &self.state {
            TileMovementState::Stopped(_) => panic!("Expected Moving state"),
            TileMovementState::Moving(state) => state.done,
        }
    }

    pub fn set_stopped(&mut self, tile_x: i16, tile_y: i16) {
        if !self.is_moving() {
            panic!("Cannot set stopped state when not moving");
        }
        self.state = TileMovementState::Stopped(TileMovementStoppedState::new(tile_x, tile_y));
    }

    pub fn set_moving(&mut self, move_dir: Direction) {
        if !self.is_stopped() {
            panic!("Cannot set moving state when not stopped");
        }
        let (current_tile_x, current_tile_y) = self.tile_position();
        self.state = TileMovementState::Moving(TileMovementMovingState::new(
            current_tile_x,
            current_tile_y,
            move_dir,
        ));
    }

    pub fn set_continue(&mut self, tile_x: i16, tile_y: i16, move_dir: Direction) {
        if !self.is_moving() {
            panic!("Cannot set continue state when not moving");
        }

        if !self.is_done() {
            panic!("Expected done state");
        }

        self.state =
            TileMovementState::Moving(TileMovementMovingState::new(tile_x, tile_y, move_dir));
    }

    // call on each tick
    pub fn process_tick(
        &mut self,
        physics: &mut PhysicsController,
    ) -> (ProcessTickResult, Option<(i16, i16)>) {
        let mut output = None;

        let mut result = match &mut self.state {
            TileMovementState::Stopped(state) => state.process_tick(),
            TileMovementState::Moving(state) => state.process_tick(physics),
        };

        if let ProcessTickResult::ShouldContinue(tile_x, tile_y, buffered_move_dir) = result {
            self.set_continue(tile_x, tile_y, buffered_move_dir);

            let (dx, dy) = buffered_move_dir.to_delta();

            let next_tile_x = tile_x + dx as i16;
            let next_tile_y = tile_y + dy as i16;

            output = Some((next_tile_x, next_tile_y));

            result = ProcessTickResult::DoNothing;
        }

        return (result, output);
    }

    // on the client, called by predicted entities
    // on the server, called by confirmed entities
    pub fn process_command(
        &mut self,
        physics: &PhysicsController,
        tick: Tick,
        command: Option<PlayerCommands>,
    ) -> Option<(i16, i16)> {
        let Some(command) = command else {
            return None;
        };
        let Some(direction) = command.get_move() else {
            return None;
        };

        // info!("process_command: {:?} {:?}", tick, direction);

        match &mut self.state {
            TileMovementState::Stopped(state) => {
                let (dx, dy) = direction.to_delta();

                let next_tile_x = state.tile_x + dx as i16;
                let next_tile_y = state.tile_y + dy as i16;

                self.set_moving(direction);

                return Some((next_tile_x, next_tile_y));
            }
            TileMovementState::Moving(state) => {
                if state.can_buffer_movement(physics) {
                    state.buffer_movement(tick, direction);
                }
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

    // call on each tick
    fn process_tick(&mut self) -> ProcessTickResult {
        return ProcessTickResult::DoNothing;
    }
}

// Tile Movement Moving State
#[derive(Clone)]
struct TileMovementMovingState {
    to_tile_x: i16,
    to_tile_y: i16,
    done: bool,

    buffered_move_dir: Option<Direction>,
}

impl TileMovementMovingState {
    fn new(from_tile_x: i16, from_tile_y: i16, move_dir: Direction) -> Self {
        let (dx, dy) = move_dir.to_delta();
        let to_tile_x = from_tile_x + dx as i16;
        let to_tile_y = from_tile_y + dy as i16;

        Self {
            to_tile_x,
            to_tile_y,
            done: false,
            buffered_move_dir: None,
        }
    }

    pub(crate) fn target_position(&self) -> Vec2 {
        return Vec2::new(
            self.to_tile_x as f32 * TILE_SIZE,
            self.to_tile_y as f32 * TILE_SIZE,
        );
    }

    // call on each tick
    fn process_tick(&mut self, physics: &mut PhysicsController) -> ProcessTickResult {
        if self.done {
            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
        }

        let target_position = self.target_position();
        let target_distance = physics.position().distance(target_position);
        const STOPPING_DISTANCE: f32 = 0.5 * TILE_SIZE;

        let target_direction = (target_position - physics.position()).normalize();

        if target_distance > STOPPING_DISTANCE {
            // speed up
            physics.speed_up(target_direction);
        } else {
            // slow down
            physics.slow_down(target_direction);
        }

        if target_distance <= physics.velocity().length() {
            // reached target!

            self.done = true;

            if self.buffered_move_dir.is_none() {
                return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
            } else {
                let buffered_move_dir = self.buffered_move_dir.take().unwrap();
                return ProcessTickResult::ShouldContinue(
                    self.to_tile_x,
                    self.to_tile_y,
                    buffered_move_dir,
                );
            }
        } else {
            physics.step();

            return ProcessTickResult::DoNothing;
        }
    }

    pub(crate) fn can_buffer_movement(&self, physics: &PhysicsController) -> bool {
        let target_position = self.target_position();
        let target_distance = physics.position().distance(target_position);
        return target_distance < TILE_SIZE * 0.5; // TODO: this should be better
    }

    pub(crate) fn buffer_movement(&mut self, tick: Tick, move_dir: Direction) {
        warn!(
            "buffering command for Tick: {:?}, MoveDir: {:?}",
            tick, move_dir
        );

        self.buffered_move_dir = Some(move_dir);
    }
}

pub enum ProcessTickResult {
    ShouldStop(i16, i16),
    ShouldContinue(i16, i16, Direction),
    DoNothing,
}
