use std::collections::VecDeque;

use bevy_ecs::prelude::Component;

use game_engine::{logging::{info, warn}, naia::Tick, world::{types::Direction, components::{NextTilePosition, ProcessTickResult, TileMovement}}};

#[derive(Component)]
pub struct ClientTileMovement {
    is_predicted: bool,
    tile_movement: TileMovement,
    buffered_future_tiles_opt: Option<VecDeque<(Tick, i16, i16)>>,
}

impl ClientTileMovement {
    pub fn new_stopped(
        predicted: bool,
        next_tile_position: &NextTilePosition,
    ) -> Self {

        let me = Self {
            is_predicted: predicted,
            tile_movement: TileMovement::new_stopped(false, predicted, next_tile_position),
            buffered_future_tiles_opt: None,
        };

        me
    }

    pub fn inner_mut(&mut self) -> &mut TileMovement {
        return &mut self.tile_movement;
    }

    // called by confirmed entities
    pub fn recv_updated_next_tile_position(
        &mut self,
        update_tick: Tick,
        next_tile_position: &NextTilePosition,
    ) {
        if self.is_predicted {
            panic!("Predicted entities do not receive updates");
        }

        let (next_tile_x, next_tile_y) = (next_tile_position.x(), next_tile_position.y());
        info!(
            "Recv NextTilePosition. Tick: {:?}, Tile: ({:?}, {:?})",
            update_tick, next_tile_x, next_tile_y
        );

        if self.tile_movement.is_stopped() {
            // is stopped

            let (current_tile_x, current_tile_y) = self.tile_movement.current_tile_position();
            if current_tile_x == next_tile_x && current_tile_y == next_tile_y {
                return;
            }

            let dx = (next_tile_x - current_tile_x) as i8;
            let dy = (next_tile_y - current_tile_y) as i8;

            if let Some(move_dir) = Direction::from_delta(dx, dy) {
                self.tile_movement.set_moving(move_dir);
            } else {
                warn!("Invalid move direction. Prev: ({:?}, {:?}), Next: ({:?}, {:?}). Buffering...", current_tile_x, current_tile_y, next_tile_x, next_tile_y);

                // buffering the invalid position, which will add pathfinding tiles to the buffer
                self.buffer_updated_next_tile_position(update_tick, next_tile_position.x(), next_tile_position.y());

                // pop buffer
                self.pop_and_use_buffered_future_tiles(current_tile_x, current_tile_y);
            }
        } else {
            // is moving
            self.buffer_updated_next_tile_position(update_tick, next_tile_position.x(), next_tile_position.y());
        }
    }

    fn pop_and_use_buffered_future_tiles(&mut self, tile_x: i16, tile_y: i16) {

        if self.tile_movement.is_stopped() {
            panic!("Buffered Future Tiles should already be used before stopping");
        }
        if !self.buffered_future_tiles_opt.is_some() {
            panic!("No buffered future tiles to pop");
        }

        let buffered_future_tiles = self.buffered_future_tiles_opt.as_mut().unwrap();

        let (next_tick, next_x, next_y) = buffered_future_tiles.pop_front().unwrap();

        warn!("Prediction({:?}), Processing Buffered Next Tile Position! Tick: {:?}, Current: ({:?}, {:?}), Next: ({:?}, {:?})", self.is_predicted, next_tick, tile_x, tile_y, next_x, next_y);

        if buffered_future_tiles.is_empty() {
            self.buffered_future_tiles_opt = None;
        }

        let dx = (next_x - tile_x) as i8;
        let dy = (next_y - tile_y) as i8;

        if let Some(move_dir) = Direction::from_delta(dx, dy) {
            self.tile_movement.set_continue(tile_x, tile_y, move_dir);
        } else {
            panic!("Invalid move direction. From: ({:?}, {:?}), To: ({:?}, {:?})", tile_x, tile_y, next_x, next_y);
        }
    }

    fn buffer_updated_next_tile_position(
        &mut self,
        updated_tick: Tick,
        next_x: i16,
        next_y: i16,
    ) {
        if self.is_predicted {
            panic!("Predicted entities do not buffer future tiles");
        }

        if self.buffered_future_tiles_opt.is_none() {
            self.buffered_future_tiles_opt = Some(VecDeque::new());
        }

        let buffered_future_tiles = self.buffered_future_tiles_opt.as_mut().unwrap();

        if let Some((_, last_x, last_y)) = buffered_future_tiles.back() {
            let last_x = *last_x;
            let last_y = *last_y;

            if last_x == next_x && last_y == next_y {
                warn!("1 Ignoring Duplicate Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                return;
            }

            if !is_valid_tile_transition(last_x, last_y, next_x, next_y, false) {
                buffer_pathfind_tiles(last_x, last_y, next_x, next_y, buffered_future_tiles);
            } else {
                warn!("1 Buffering Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                buffered_future_tiles.push_back((updated_tick, next_x, next_y));
            }
        } else {

            let (last_x, last_y) = if self.tile_movement.is_stopped() {
                self.tile_movement.current_tile_position()
            } else {
                self.tile_movement.to_tile_position()
            };

            if last_x == next_x && last_y == next_y {
                warn!("2 Ignoring Duplicate Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                return;
            }

            if !is_valid_tile_transition(last_x, last_y, next_x, next_y, false) {
                buffer_pathfind_tiles(last_x, last_y, next_x, next_y, buffered_future_tiles);
            } else {
                warn!("2 Buffering Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                buffered_future_tiles.push_back((updated_tick, next_x, next_y));
            }
        }
    }

    // called by predicted entities
    pub fn recv_rollback(&mut self, confirmed_tile_movement: &ClientTileMovement) {
        if !self.is_predicted {
            panic!("Only predicted entities can receive rollbacks");
        }
        if confirmed_tile_movement.is_predicted {
            panic!("Predicted entities cannot send rollbacks");
        }

        self.tile_movement.mirror(&confirmed_tile_movement.tile_movement);
        self.buffered_future_tiles_opt = confirmed_tile_movement.buffered_future_tiles_opt.clone();
    }

    pub fn finish_rollback(&mut self) {
        if !self.is_predicted {
            panic!("Only predicted entities can finish rollbacks");
        }
        self.buffered_future_tiles_opt = None;
    }

    pub fn process_result(&mut self, result: ProcessTickResult) {
        match result {
            ProcessTickResult::ShouldStop(tile_x, tile_y) => {

                if self.buffered_future_tiles_opt.is_some() {
                    self.pop_and_use_buffered_future_tiles(tile_x, tile_y);
                } else {
                    self.tile_movement.set_stopped(tile_x, tile_y);
                }
            }
            ProcessTickResult::DoNothing => {},
            ProcessTickResult::ShouldContinue(_, _, _) => {
                panic!("ShouldMove not expected");
            }
        }
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

fn is_valid_tile_transition(ax: i16, ay: i16, bx: i16, by: i16, prediction: bool) -> bool {
    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    let d_dis = dx + dy;
    if d_dis == 0 || d_dis > 2 || dx > 1 || dy > 1 {
        warn!(
            "from_tile and to_tile are not adjacent. ({:?}, {:?}) -> ({:?}, {:?}). Prediction: {:?}",
            ax, ay, bx, by, prediction,
        );
        return false;
    } else {
        return true;
    }
}