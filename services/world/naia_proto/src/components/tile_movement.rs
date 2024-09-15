
use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Instant, Tick};

use crate::{components::NextTilePosition, messages::KeyCommand, constants::{MOVEMENT_SPEED, TILE_SIZE}};

#[derive(Component)]
pub struct TileMovement {
    is_server: bool,
    is_predicted: bool,

    state: TileMovementState,
    outbound_next_tile: Option<(i16, i16)>,
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
        let mut me = Self {
            is_server,
            is_predicted: predicted,

            state: TileMovementState::stopped(next_tile_position.x(), next_tile_position.y()),
            outbound_next_tile: None,
        };

        me
    }

    // retrieve the current position of the entity
    pub fn current_position(&self) -> (f32, f32) {
        match &self.state {
            TileMovementState::Stopped(state) => state.current_position(),
            TileMovementState::Moving(state) => state.current_position(),
        }
    }

    // return whether the entity is moving
    pub fn is_moving(&self) -> bool {
        self.state.is_moving()
    }

    // return rotation of the entity if moving
    pub fn get_direction(&self) -> Option<f32> {
        if !self.state.is_moving() {
            return None;
        }

        let TileMovementState::Moving(state) = &self.state else {
            panic!("Expected Moving state");
        };

        let from_x = state.from_tile_x;
        let from_y = state.from_tile_y;
        let to_x = state.to_tile_x;
        let to_y = state.to_tile_y;

        let dis_x = to_x - from_x;
        let dis_y = to_y - from_y;

        if dis_x == 0 && dis_y == -1 {
            return Some(0.0 + 90.0);
        }
        if dis_x == 1 && dis_y == -1 {
            return Some(45.0 + 90.0);
        }
        if dis_x == 1 && dis_y == 0 {
            return Some(90.0 + 90.0);
        }
        if dis_x == 1 && dis_y == 1 {
            return Some(135.0 + 90.0);
        }
        if dis_x == 0 && dis_y == 1 {
            return Some(180.0 + 90.0);
        }
        if dis_x == -1 && dis_y == 1 {
            return Some(225.0 + 90.0);
        }
        if dis_x == -1 && dis_y == 0 {
            return Some(270.0 + 90.0);
        }
        if dis_x == -1 && dis_y == -1 {
            return Some(315.0 + 90.0);
        }

        None
    }

    // on the client, called by predicted entities
    // on the server, called by confirmed entities
    pub fn recv_command(&mut self, key_command: &KeyCommand) {
        if !self.is_server && !self.is_predicted {
            panic!("Only predicted entities can receive commands");
        }

        if !key_command.a && !key_command.d && !key_command.w && !key_command.s {
            return;
        }

        if self.state.is_moving() {
            return;
        }

        let TileMovementState::Stopped(state) = &self.state else {
            panic!("Expected Stopped state");
        };

        let current_tile_x = state.tile_x;
        let current_tile_y = state.tile_y;
        let mut next_tile_x = state.tile_x;
        let mut next_tile_y = state.tile_y;

        if key_command.w {
            next_tile_y -= 1;
        }
        if key_command.s {
            next_tile_y += 1;
        }
        if key_command.a {
            next_tile_x -= 1;
        }
        if key_command.d {
            next_tile_x += 1;
        }

        self.state = TileMovementState::moving(
            current_tile_x,
            current_tile_y,
            next_tile_x,
            next_tile_y,
            0.0,
        );

        if self.is_server {
            self.outbound_next_tile = Some((next_tile_x, next_tile_y));
        }
    }

    // on the client, called by confirmed entities
    // on the server, never called
    pub fn recv_updated_next_tile_position(&mut self, update_tick: Tick, next_tile_position: &NextTilePosition) {
        if self.is_server {
            panic!("Server entities do not receive updates");
        }
        if self.is_predicted {
            panic!("Predicted entities do not receive updates");
        }

        if self.state.is_stopped() {
            // is stopped
            let TileMovementState::Stopped(state) = &self.state else {
                panic!("Expected Stopped state");
            };

            let (current_tile_x, current_tile_y) = (state.tile_x, state.tile_y);
            let (next_tile_x, next_tile_y) = (next_tile_position.x(), next_tile_position.y());
            if current_tile_x == next_tile_x && current_tile_y == next_tile_y {
                return;
            }
            self.state = TileMovementState::moving(
                current_tile_x,
                current_tile_y,
                next_tile_x,
                next_tile_y,
                0.0,
            );

        } else {
            // is moving
            let TileMovementState::Moving(state) = &mut self.state else {
                panic!("Expected Moving state");
            };

            state.recv_updated_next_tile_position(update_tick, next_tile_position.x(), next_tile_position.y());
        }
    }

    // on the client, never called
    // on the server, called by confirmed entities
    pub fn send_updated_next_tile_position(&mut self, next_tile_position: &mut NextTilePosition) {
        if !self.is_server {
            panic!("Client entities do not send updates");
        }
        if let Some((next_tile_x, next_tile_y)) = self.outbound_next_tile.take() {
            next_tile_position.set(next_tile_x, next_tile_y);
        }
    }

    // on the client, called by predicted entities
    // on the server, never called
    pub fn recv_rollback(&mut self, server_tile_movement: &TileMovement) {
        if self.is_server {
            panic!("Server entities do not receive rollbacks");
        }
        if !self.is_predicted {
            panic!("Only predicted entities can receive rollbacks");
        }
        if server_tile_movement.is_server {
            panic!("Server entities cannot send rollbacks");
        }
        if server_tile_movement.is_predicted {
            panic!("Predicted entities cannot send rollbacks");
        }

        self.state = server_tile_movement.state.clone();
    }

    // call on each tick
    pub fn process_tick(&mut self) {

        let result = match &mut self.state {
            TileMovementState::Stopped(state) => state.process_tick(),
            TileMovementState::Moving(state) => state.process_tick(),
        };

        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                self.state = TileMovementState::stopped(tile_x, tile_y);
            }
            ProcessTickResult::DoNothing => {}
        }
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

    fn moving(ax: i16, ay: i16, bx: i16, by: i16, interp: f32) -> Self {
        Self::Moving(TileMovementMovingState::new(
            ax, ay, bx, by, interp,
        ))
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
}

// Tile Movement Stopped State
#[derive(Clone)]
struct TileMovementStoppedState {
    tile_x: i16,
    tile_y: i16,
}

impl TileMovementStoppedState {
    fn new(tile_x: i16, tile_y: i16) -> Self {
        Self {
            tile_x,
            tile_y,
        }
    }

    // retrieve the current position of the entity
    fn current_position(&self) -> (f32, f32) {
        (self.tile_x as f32 * TILE_SIZE, self.tile_y as f32 * TILE_SIZE)
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
    distance: f32,
    distance_max: f32,
    done: bool,
}

impl TileMovementMovingState {
    fn new(ax: i16, ay: i16, bx: i16, by: i16, interp: f32) -> Self {
        if ax == bx && ay == by {
            panic!("from_tile and to_tile are the same");
        }
        if interp < 0.0 || interp > 1.0 {
            panic!("interp must be between 0.0 and 1.0");
        }
        if (ax - bx).abs() + (ay - by).abs() > 2 {
            panic!("from_tile and to_tile are not adjacent");
        }
        let x_axis_changed: bool = ax != bx;
        let y_axis_changed: bool = ay != by;

        let distance_max = if x_axis_changed && y_axis_changed {
            std::f32::consts::SQRT_2
        } else {
            1.0
        };
        let distance_max = distance_max * TILE_SIZE;
        let distance = distance_max * interp;

        let done = interp == 1.0;

        Self {
            from_tile_x: ax,
            from_tile_y: ay,
            to_tile_x: bx,
            to_tile_y: by,
            distance,
            distance_max,
            done,
        }
    }

    // retrieve the current position of the entity
    fn current_position(&self) -> (f32, f32) {
        let interp = self.distance / self.distance_max; // this is what is varying across frames

        let from_x = self.from_tile_x as f32 * TILE_SIZE;
        let from_y = self.from_tile_y as f32 * TILE_SIZE;
        let to_x = self.to_tile_x as f32 * TILE_SIZE;
        let to_y = self.to_tile_y as f32 * TILE_SIZE;

        let dis_x = to_x - from_x;
        let dis_y = to_y - from_y;

        (from_x + (dis_x * interp), from_y + (dis_y * interp))
    }

    // call on each tick
    fn process_tick(&mut self) -> ProcessTickResult {

        if self.done {
            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
        }

        self.distance += MOVEMENT_SPEED;

        if self.distance >= self.distance_max {
            self.distance = self.distance_max;
            self.done = true;
            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
        } else {
            return ProcessTickResult::DoNothing;
        }
    }

    pub(crate) fn recv_updated_next_tile_position(&mut self, updated_tick: Tick, _next_x: i16, _next_y: i16) {
        // TODO: still need to implement this!
        panic!("recv_updated_next_tile_position(). updated tick: {:?}", updated_tick);
    }
}

enum ProcessTickResult {
    ShouldStop(i16, i16),
    DoNothing,
}