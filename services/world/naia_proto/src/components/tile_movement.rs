use naia_bevy_shared::Tick;

use math::Vec2;

use crate::{
    components::{MoveBuffer, NextTilePosition, PhysicsController},
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

    pub fn is_prediction(&self) -> bool {
        match self {
            TileMovementType::Server => false,
            TileMovementType::ClientConfirmed => false,
            TileMovementType::ClientPredicted => true,
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

    pub fn set_tile_position(&mut self, tx: i16, ty: i16) {
        match &mut self.state {
            TileMovementState::Stopped(state) => {
                state.tile_x = tx;
                state.tile_y = ty;
            }
            TileMovementState::Moving(state) => {
                state.to_tile_x = tx;
                state.to_tile_y = ty;
            }
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
        let stopped_state = self.as_stopped();
        self.state = TileMovementState::Moving(TileMovementMovingState::new(
            stopped_state.tile_x,
            stopped_state.tile_y,
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
        has_future: bool,
        physics: &mut PhysicsController,
        tick: Tick,
        is_prediction: bool,
    ) -> ProcessTickResult {
        let output = match &mut self.state {
            TileMovementState::Stopped(state) => state.process_tick(),
            TileMovementState::Moving(state) => state.process_tick(has_future, physics),
        };

        physics.tick_log(tick, is_prediction);

        output
    }

    // on the client, called by predicted entities
    // on the server, called by confirmed entities
    pub fn process_command(
        &mut self,
        physics: &PhysicsController,
        move_buffer: &mut MoveBuffer,
        tick: Tick,
        command: Option<PlayerCommands>,
    ) -> (Option<(i16, i16)>, Option<Option<Direction>>) {
        let Some(command) = command else {
            return (None, None);
        };
        let Some(direction) = command.get_move() else {
            return (None, None);
        };

        // info!("process_command: {:?} {:?}", tick, direction);

        match &mut self.state {
            TileMovementState::Stopped(state) => {
                let (dx, dy) = direction.to_delta();

                let next_tile_x = state.tile_x + dx as i16;
                let next_tile_y = state.tile_y + dy as i16;

                self.set_moving(direction);

                return (Some((next_tile_x, next_tile_y)), None);
            }
            TileMovementState::Moving(state) => {
                if state.can_buffer_movement(physics) {
                    let prev_move = move_buffer.buffered_move();

                    state.buffer_movement(move_buffer, tick, direction);

                    if prev_move != Some(direction) {
                        return (None, Some(Some(direction)));
                    }
                }

                return (None, None);
            }
        }
    }

    pub fn mirror(&mut self, other: &TileMovement) {
        self.state = other.state.clone();
    }

    pub fn as_stopped(&self) -> &TileMovementStoppedState {
        match &self.state {
            TileMovementState::Stopped(state) => state,
            TileMovementState::Moving(_) => panic!("Expected Stopped state"),
        }
    }

    pub fn as_stopped_mut(&mut self) -> &mut TileMovementStoppedState {
        match &mut self.state {
            TileMovementState::Stopped(state) => state,
            TileMovementState::Moving(_) => panic!("Expected Stopped state"),
        }
    }

    pub fn as_moving(&self) -> &TileMovementMovingState {
        match &self.state {
            TileMovementState::Stopped(_) => panic!("Expected Moving state"),
            TileMovementState::Moving(state) => state,
        }
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
pub struct TileMovementStoppedState {
    tile_x: i16,
    tile_y: i16,
}

impl TileMovementStoppedState {
    fn new(tile_x: i16, tile_y: i16) -> Self {
        // info!("New Stopped State: ({:?}, {:?})", tile_x, tile_y);
        Self { tile_x, tile_y }
    }

    // call on each tick
    fn process_tick(&mut self) -> ProcessTickResult {
        return ProcessTickResult::DoNothing;
    }

    pub fn tile_position(&self) -> (i16, i16) {
        (self.tile_x, self.tile_y)
    }

    pub fn set_tile_position(&mut self, tx: i16, ty: i16) {
        self.tile_x = tx;
        self.tile_y = ty;
    }
}

// Tile Movement Moving State
#[derive(Clone)]
pub struct TileMovementMovingState {
    from_tile_x: i16,
    from_tile_y: i16,
    to_tile_x: i16,
    to_tile_y: i16,
    done: bool,
}

impl TileMovementMovingState {
    fn new(from_tile_x: i16, from_tile_y: i16, move_dir: Direction) -> Self {
        let (dx, dy) = move_dir.to_delta();
        let to_tile_x = from_tile_x + dx as i16;
        let to_tile_y = from_tile_y + dy as i16;

        // info!("New Moving State. From ({:?}, {:?}) to ({:?}, {:?})", from_tile_x, from_tile_y, to_tile_x, to_tile_y);

        Self {
            from_tile_x,
            from_tile_y,
            to_tile_x,
            to_tile_y,
            done: false,
        }
    }

    pub(crate) fn target_position(&self) -> Vec2 {
        return Vec2::new(
            self.to_tile_x as f32 * TILE_SIZE,
            self.to_tile_y as f32 * TILE_SIZE,
        );
    }

    pub fn tile_positions(&self) -> (i16, i16, i16, i16) {
        (
            self.from_tile_x,
            self.from_tile_y,
            self.to_tile_x,
            self.to_tile_y,
        )
    }

    pub fn target_tile_position(&self) -> (i16, i16) {
        (self.to_tile_x, self.to_tile_y)
    }

    // call on each tick
    fn process_tick(
        &mut self,
        has_future: bool,
        physics: &mut PhysicsController,
    ) -> ProcessTickResult {
        if self.done {
            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
        }

        let target_position = self.target_position();
        let target_distance = physics.position().distance(target_position);
        const STOPPING_DISTANCE: f32 = 0.5 * TILE_SIZE;

        let target_direction = (target_position - physics.position()).normalize();

        if target_distance > STOPPING_DISTANCE || has_future {
            // speed up
            physics.speed_up(target_direction);
        } else {
            // slow down
            physics.slow_down(target_direction);
        }

        if target_distance <= physics.velocity().length() {
            // reached target!

            self.done = true;

            physics.set_tile_position(self.to_tile_x, self.to_tile_y, false);

            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
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

    pub(crate) fn buffer_movement(
        &mut self,
        move_buffer: &mut MoveBuffer,
        _tick: Tick,
        move_dir: Direction,
    ) {
        // info!(
        //     "buffering command for Tick: {:?}, MoveDir: {:?}",
        //     tick, move_dir
        // );

        move_buffer.buffer_move(move_dir);
    }
}

#[derive(Clone, Copy)]
pub enum ProcessTickResult {
    ShouldStop(i16, i16),
    DoNothing,
}
