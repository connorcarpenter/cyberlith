
use bevy_ecs::prelude::Component;

use game_engine::{
    logging::{info, warn},
    naia::Tick,
    world::{
        components::{NextTilePosition, MoveBuffer, ProcessTickResult, TileMovement},
        types::Direction,
    },
};

use crate::components::{client_tile_movement::ClientTileMovement};

#[derive(Component, Clone)]
pub struct ConfirmedTileMovement {
    pub(crate) tile_movement: TileMovement,
}

impl ClientTileMovement for ConfirmedTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, Option<&mut MoveBuffer>) {
        (&mut self.tile_movement, None)
    }

    fn process_result(&mut self, result: ProcessTickResult) {

        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {
                self.tile_movement.set_stopped(tile_x, tile_y);
            }
            ProcessTickResult::DoNothing => {}
        }
    }

    fn has_future(&self) -> bool {
        false
    }
}

impl ConfirmedTileMovement {
    pub fn new_stopped(next_tile_position: &NextTilePosition) -> Self {
        Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
        }
    }

    // called by confirmed entities
    pub fn recv_updated_next_tile_position(
        &mut self,
        update_tick: Tick,
        next_tile_position: &NextTilePosition,
    ) {
        let (next_tile_x, next_tile_y) = (next_tile_position.x(), next_tile_position.y());
        info!(
            "Recv NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
            update_tick, next_tile_x, next_tile_y
        );

        let (current_tile_x, current_tile_y) = self.tile_movement.tile_position();

        if current_tile_x == next_tile_x && current_tile_y == next_tile_y {
            return;
        }

        if self.tile_movement.is_moving() {
            self.tile_movement.set_stopped(current_tile_x, current_tile_y);
        }

        let dx = (next_tile_x - current_tile_x) as i8;
        let dy = (next_tile_y - current_tile_y) as i8;

        if let Some(move_dir) = Direction::from_delta(dx, dy) {
            self.tile_movement.set_moving(move_dir);
        } else {
            warn!(
                "Invalid move direction. Prev: ({:?}, {:?}), Next: ({:?}, {:?}). Pathfinding...",
                current_tile_x, current_tile_y, next_tile_x, next_tile_y
            );

            let (last_x, last_y, move_dir) = pathfind_to_tile(current_tile_x, current_tile_y, next_tile_x, next_tile_y);

            self.tile_movement.set_tile_position(last_x, last_y);
            self.tile_movement.set_moving(move_dir);
        }
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