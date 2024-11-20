
use bevy_ecs::prelude::Component;

use game_engine::{
    logging::{info, warn},
    naia::Tick,
    world::{
        components::{PhysicsController, NextTilePosition, HasMoveBuffered, MoveBuffer, ProcessTickResult, TileMovement},
        types::Direction,
    },
};

use crate::{resources::TickTracker, components::{client_tile_movement::ClientTileMovement, RenderPosition}};

#[derive(Component, Clone)]
pub struct ConfirmedTileMovement {
    tile_movement: TileMovement,
    move_buffer: MoveBuffer,
}

impl ClientTileMovement for ConfirmedTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, &mut MoveBuffer) {
        (&mut self.tile_movement, &mut self.move_buffer)
    }

    fn process_result(&mut self, result: ProcessTickResult) {

        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                if self.move_buffer.has_buffered_move() {
                    let buffered_move_dir = self.move_buffer.pop_buffered_move().unwrap();
                    self.tile_movement.set_continue(tile_x, tile_y, buffered_move_dir);
                } else {
                    self.tile_movement.set_stopped(tile_x, tile_y);
                }
            }
            ProcessTickResult::DoNothing => {}
        }
    }
}

impl ConfirmedTileMovement {
    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            move_buffer: MoveBuffer::new(),
        }
    }

    pub fn recv_updated_next_tile_position(
        &mut self,
        tick_tracker: &TickTracker,
        mut update_tick: Tick,
        next_tile_position: &NextTilePosition,
        physics: &mut PhysicsController,
        render_position: &mut RenderPosition,
    ) {
        let (next_tile_x, next_tile_y) = (next_tile_position.x(), next_tile_position.y());

        let (current_tile_x, current_tile_y) = self.tile_movement.tile_position();

        info!(
            "Recv NextTilePosition. Tick: {:?}, Current Tile: ({:?}, {:?}), Next Tile: ({:?}, {:?}), Velocity: ({:?}, {:?})",
            update_tick, current_tile_x, current_tile_y, next_tile_x, next_tile_y, next_tile_position.velocity_x(), next_tile_position.velocity_y()
        );

        physics.set_velocity(next_tile_position.velocity_x(), next_tile_position.velocity_y());

        if current_tile_x == next_tile_x && current_tile_y == next_tile_y {
            if self.tile_movement.is_stopped() {
                panic!("Unexpected! Current tile position is the same as the next tile position");
            } else {
                // already processed command

                let (last_x, last_y) = self.tile_movement.reset_movement();
                physics.set_tile_position(last_x, last_y, true);
            }
        } else {

            if self.tile_movement.is_moving() {
                self.tile_movement.set_stopped(current_tile_x, current_tile_y);
            }

            let dx = (next_tile_x - current_tile_x) as i8;
            let dy = (next_tile_y - current_tile_y) as i8;

            if let Some(move_dir) = Direction::from_delta(dx, dy) {

                physics.set_tile_position(current_tile_x, current_tile_y, true);

                self.tile_movement.set_moving(move_dir);
            } else {
                panic!( // revert this to a warn! when link conditioner has jitter/loss again
                    "Invalid move direction. Prev: ({:?}, {:?}), Next: ({:?}, {:?}). Pathfinding...",
                    current_tile_x, current_tile_y, next_tile_x, next_tile_y
                );

                let (last_x, last_y, move_dir) = pathfind_to_tile(current_tile_x, current_tile_y, next_tile_x, next_tile_y);

                self.tile_movement.set_tile_position(last_x, last_y);
                physics.set_tile_position(last_x, last_y, true);
                self.tile_movement.set_moving(move_dir);
            }
        }

        if let Some(last_tick) = tick_tracker.last_processed_server_tick() {
            update_tick = last_tick;
        }
        render_position.recv_position(physics.position(), update_tick);
    }

    pub fn recv_updated_has_move_buffered(
        &mut self,
        update_tick: Tick,
        has_move_buffered: &HasMoveBuffered,
    ) {
        info!(
            "Recv HasMoveBuffered. Tick: {:?}, HasMoveBuffered: {:?}",
            update_tick, has_move_buffered.buffered()
        );
        if let Some(has_move_buffered) = has_move_buffered.buffered() {
            self.move_buffer.buffer_move(has_move_buffered);
        } else {
            self.move_buffer.clear();
        }
    }

    pub fn decompose_to_values(self) -> (TileMovement, MoveBuffer) {
        (self.tile_movement, self.move_buffer)
    }
}

fn pathfind_to_tile(
    ax: i16,
    ay: i16,
    bx: i16,
    by: i16,
) -> (i16, i16, Direction) {
    info!(
        "Pathfinding from ({:?}, {:?}) to ({:?}, {:?})",
        ax, ay, bx, by
    );

    let mut lx = ax;
    let mut ly = ay;
    let mut dir = None;

    let mut cx = ax;
    let mut cy = ay;

    while cx != bx || cy != by {
        let dx = (bx - cx).min(1).max(-1);
        let dy = (by - cy).min(1).max(-1);

        dir = Direction::from_delta(dx as i8, dy as i8);
        if dir.is_none() {
            panic!("unexpected! shouldn't be allowed");
        }

        lx = cx;
        ly = cy;

        cx += dx;
        cy += dy;

        info!("Pathfinding: ({:?}, {:?})", cx, cy);
    }

    (lx, ly, dir.unwrap())
}