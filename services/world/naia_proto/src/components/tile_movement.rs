use std::collections::VecDeque;

use bevy_ecs::prelude::Component;

use naia_bevy_shared::Tick;
use input::Key;
use logging::{info, warn};

use crate::{
    components::NextTilePosition,
    constants::{MOVEMENT_SPEED, TILE_SIZE},
    resources::KeyEvent,
};

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
        let me = Self {
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

        let dis_x = (to_x - from_x).min(1).max(-1);
        let dis_y = (to_y - from_y).min(1).max(-1);

        if dis_x == 0 && dis_y == 0 {
            return None;
        }

        let angle_radians = (dis_y as f32).atan2(dis_x as f32);
        let angle_degrees = angle_radians.to_degrees();
        let angle_degrees = (angle_degrees + 90.0) % 360.0;
        Some(angle_degrees)
    }

    // on the client, called by predicted entities
    // on the server, called by confirmed entities
    pub fn recv_command(&mut self, key_events: Vec<KeyEvent>, prediction: bool) {
        if !self.is_server && !self.is_predicted {
            panic!("Only predicted entities can receive commands");
        }

        let mut dx = 0;
        let mut dy = 0;

        let mut w = 0;
        let mut s = 0;
        let mut a = 0;
        let mut d = 0;

        for key_event in key_events {

            match key_event {
                KeyEvent::Pressed(key, duration) => {
                    if duration > 150 {
                        // hold
                        match key {
                            Key::W => w = 2,
                            Key::S => s = 2,
                            Key::A => a = 2,
                            Key::D => d = 2,
                            _ => {}
                        }
                    } else {
                        // tap
                        match key {
                            Key::W => if w == 0 { w = 1},
                            Key::S => if s == 0 { s = 1},
                            Key::A => if a == 0 { a = 1},
                            Key::D => if d == 0 { d = 1},
                            _ => {}
                        }
                    }
                }
                KeyEvent::Held(key, duration) => {
                    if duration > 150 {
                        // hold
                        match key {
                            Key::W => w = 2,
                            Key::S => s = 2,
                            Key::A => a = 2,
                            Key::D => d = 2,
                            _ => {}
                        }
                    } else {
                        // tap
                        match key {
                            Key::W => if w == 0 { w = 1},
                            Key::S => if s == 0 { s = 1},
                            Key::A => if a == 0 { a = 1},
                            Key::D => if d == 0 { d = 1},
                            _ => {}
                        }
                    }
                }
                KeyEvent::Released(key) => {

                }
            }
        }

        if w == 2 {
            dy -= 1;
        }
        if s == 2 {
            dy += 1;
        }
        if a == 2 {
            dx -= 1;
        }
        if d == 2 {
            dx += 1;
        }

        // diagonals
        if dx != 0 && dy == 0 {
            if w == 1 {
                dy -= 1;
            }
            if s == 1 {
                dy += 1;
            }
        }
        if dy != 0 && dx == 0 {
            if a == 1 {
                dx -= 1;
            }
            if d == 1 {
                dx += 1;
            }
        }

        if dx == 0 && dy == 0 {
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
        let next_tile_x = state.tile_x + dx;
        let next_tile_y = state.tile_y + dy;

        self.state = TileMovementState::moving(
            current_tile_x,
            current_tile_y,
            next_tile_x,
            next_tile_y,
            prediction,
        );

        if self.is_server {
            self.outbound_next_tile = Some((next_tile_x, next_tile_y));
        }
    }

    // on the client, called by confirmed entities
    // on the server, never called
    pub fn recv_updated_next_tile_position(
        &mut self,
        update_tick: Tick,
        next_tile_position: &NextTilePosition,
        prediction: bool,
    ) {
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

            // info!(
            //     "Recv NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
            //     update_tick, next_tile_x, next_tile_y
            // );

            self.state = TileMovementState::moving(
                current_tile_x,
                current_tile_y,
                next_tile_x,
                next_tile_y,
                prediction,
            );
        } else {
            // is moving
            let TileMovementState::Moving(state) = &mut self.state else {
                panic!("Expected Moving state");
            };

            state.recv_updated_next_tile_position(
                update_tick,
                next_tile_position.x(),
                next_tile_position.y(),
                self.is_predicted,
            );
        }
    }

    // on the client, never called
    // on the server, called by confirmed entities
    pub fn send_updated_next_tile_position(
        &mut self,
        tick: Tick,
        next_tile_position: &mut NextTilePosition,
    ) {
        if !self.is_server {
            panic!("Client entities do not send updates");
        }
        if let Some((next_tile_x, next_tile_y)) = self.outbound_next_tile.take() {
            next_tile_position.set(next_tile_x, next_tile_y);

            info!(
                "Send NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
                tick, next_tile_x, next_tile_y
            );
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
            TileMovementState::Moving(state) => state.process_tick(self.is_predicted),
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

    fn moving(ax: i16, ay: i16, bx: i16, by: i16, prediction: bool) -> Self {
        Self::Moving(TileMovementMovingState::new(ax, ay, bx, by, prediction))
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
        Self { tile_x, tile_y }
    }

    // retrieve the current position of the entity
    fn current_position(&self) -> (f32, f32) {
        (
            self.tile_x as f32 * TILE_SIZE,
            self.tile_y as f32 * TILE_SIZE,
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
    distance: f32,
    distance_max: f32,
    done: bool,
    buffered_future_tiles_opt: Option<VecDeque<(Tick, i16, i16)>>,
}

impl TileMovementMovingState {
    fn new(ax: i16, ay: i16, bx: i16, by: i16, prediction: bool) -> Self {

        if ax == bx && ay == by {
            panic!("from_tile and to_tile are the same");
        }

        let mut to_tile_x = bx;
        let mut to_tile_y = by;

        let mut buffered_future_tiles_opt: Option<VecDeque<(Tick, i16, i16)>> = None;
        if !is_valid_tile_transition(ax, ay, bx, by, prediction) {
            buffered_future_tiles_opt = Some(VecDeque::new());
            let mut buffered_future_tiles = buffered_future_tiles_opt.as_mut().unwrap();
            buffer_pathfind_tiles(ax, ay, bx, by, &mut buffered_future_tiles);
            buffered_future_tiles.push_back((0, bx, by));
            let (_, next_x, next_y) = buffered_future_tiles.pop_front().unwrap();
            to_tile_x = next_x;
            to_tile_y = next_y;
        }

        Self {
            from_tile_x: ax,
            from_tile_y: ay,
            to_tile_x,
            to_tile_y,
            distance: 0.0,
            distance_max: get_tile_distance(ax, ay, to_tile_x, to_tile_y),
            done: false,
            buffered_future_tiles_opt,
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
    fn process_tick(&mut self, prediction: bool,) -> ProcessTickResult {
        if self.done {
            return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
        }

        self.distance += MOVEMENT_SPEED;

        if self.distance >= self.distance_max {

            if self.buffered_future_tiles_opt.is_none() {
                self.distance = self.distance_max;
                self.done = true;
                return ProcessTickResult::ShouldStop(self.to_tile_x, self.to_tile_y);
            } else {
                // we have buffered future tiles
                self.distance -= self.distance_max;

                let buffered_future_tiles = self.buffered_future_tiles_opt.as_mut().unwrap();
                let (next_tick, next_x, next_y) = buffered_future_tiles.pop_front().unwrap();

                warn!("Prediction({:?}), Processing Buffered Next Tile Position! Tick: {:?}, Tile: ({:?}, {:?})", prediction, next_tick, next_x, next_y);

                if buffered_future_tiles.is_empty() {
                    self.buffered_future_tiles_opt = None;
                }

                self.from_tile_x = self.to_tile_x;
                self.from_tile_y = self.to_tile_y;
                self.to_tile_x = next_x;
                self.to_tile_y = next_y;

                self.distance_max = get_tile_distance(self.from_tile_x, self.from_tile_y, self.to_tile_x, self.to_tile_y);
                return ProcessTickResult::DoNothing;
            }
        } else {
            return ProcessTickResult::DoNothing;
        }
    }

    pub(crate) fn recv_updated_next_tile_position(
        &mut self,
        updated_tick: Tick,
        next_x: i16,
        next_y: i16,
        prediction: bool,
    ) {
        if self.buffered_future_tiles_opt.is_none() {
            self.buffered_future_tiles_opt = Some(VecDeque::new());
        }

        let buffered_future_tiles = self.buffered_future_tiles_opt.as_mut().unwrap();

        if let Some((_, last_x, last_y)) = buffered_future_tiles.back() {
            let last_x = *last_x;
            let last_y = *last_y;

            if last_x == next_x && last_y == next_y {
                warn!("Prediction({:?}), Ignoring Duplicate Next Tile Position! Tick: {:?}, Tile: ({:?}, {:?})", prediction, updated_tick, next_x, next_y);
                return;
            }

            if !is_valid_tile_transition(last_x, last_y, next_x, next_y, prediction) {
                buffer_pathfind_tiles(last_x, last_y, next_x, next_y, buffered_future_tiles);
            }
        }

        buffered_future_tiles.push_back((updated_tick, next_x, next_y));
        warn!("Prediction({:?}), Buffering Next Tile Position! Tick: {:?}, Tile: ({:?}, {:?})", prediction, updated_tick, next_x, next_y);
    }
}

// does not add (ax, ay) or (bx, by) to the buffer
fn buffer_pathfind_tiles(
    ax: i16, ay: i16,
    bx: i16, by: i16,
    buffer: &mut VecDeque<(Tick, i16, i16)>
) {
    info!("Pathfinding from ({:?}, {:?}) to ({:?}, {:?})", ax, ay, bx, by);

    let mut cx = ax;
    let mut cy = ay;

    while cx != bx || cy != by {
        let dx = (bx - cx).min(1).max(-1);
        let dy = (by - cy).min(1).max(-1);

        cx += dx;
        cy += dy;

        info!("Pathfinding: ({:?}, {:?})", cx, cy);
        buffer.push_back((0, cx, cy));
    }
}

enum ProcessTickResult {
    ShouldStop(i16, i16),
    DoNothing,
}

fn get_tile_distance(ax: i16, ay: i16, bx: i16, by: i16) -> f32 {
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

fn is_valid_tile_transition(ax: i16, ay: i16, bx: i16, by: i16, prediction: bool) -> bool {
    if (ax - bx).abs() + (ay - by).abs() > 2 {
        warn!(
            "from_tile and to_tile are not adjacent. ({:?}, {:?}) -> ({:?}, {:?}). Prediction: {:?}",
            ax, ay, bx, by, prediction,
        );
        return false;
    } else {
        return true;
    }
}