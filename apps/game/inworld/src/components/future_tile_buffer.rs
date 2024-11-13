use std::collections::VecDeque;

use game_engine::{world::{components::TileMovement, types::Direction}, naia::Tick, logging::{info, warn}};

#[derive(Clone)]
pub struct FutureTileBuffer {
    buffered_future_tiles_opt: Option<VecDeque<(Tick, i16, i16)>>,
}

impl FutureTileBuffer {
    pub fn new() -> Self {
        Self {
            buffered_future_tiles_opt: None,
        }
    }

    pub fn has_tiles(&self) -> bool {
        self.buffered_future_tiles_opt.is_some()
    }

    pub fn buffer_updated_next_tile_position(&mut self, updated_tick: Tick, last_x: i16, last_y: i16, next_x: i16, next_y: i16) {
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

            if !is_valid_tile_transition(last_x, last_y, next_x, next_y) {
                buffer_pathfind_tiles(last_x, last_y, next_x, next_y, buffered_future_tiles);
            } else {
                warn!("1 Buffering Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                buffered_future_tiles.push_back((updated_tick, next_x, next_y));
            }
        } else {
            if last_x == next_x && last_y == next_y {
                warn!("2 Ignoring Duplicate Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                return;
            }

            if !is_valid_tile_transition(last_x, last_y, next_x, next_y) {
                buffer_pathfind_tiles(last_x, last_y, next_x, next_y, buffered_future_tiles);
            } else {
                warn!("2 Buffering Next Tile Position! Tick: {:?}, Last: ({:?}, {:?}), Next: ({:?}, {:?})", updated_tick, last_x, last_y, next_x, next_y);
                buffered_future_tiles.push_back((updated_tick, next_x, next_y));
            }
        }
    }

    pub fn pop_and_use_buffered_future_tiles(&mut self, tile_movement: &mut TileMovement, tile_x: i16, tile_y: i16) {

        let (_next_tick, next_x, next_y) = self.pop_future_tile(tile_x, tile_y);

        let dx = (next_x - tile_x) as i8;
        let dy = (next_y - tile_y) as i8;

        if let Some(move_dir) = Direction::from_delta(dx, dy) {
            if tile_movement.is_stopped() {
                tile_movement.set_moving(move_dir);
            } else {
                tile_movement.set_continue(tile_x, tile_y, move_dir);
            }
        } else {
            panic!(
                "Invalid move direction. From: ({:?}, {:?}), To: ({:?}, {:?})",
                tile_x, tile_y, next_x, next_y
            );
        }
    }

    fn pop_future_tile(&mut self, tile_x: i16, tile_y: i16) -> (Tick, i16, i16) {
        if self.buffered_future_tiles_opt.is_none() {
            panic!("No buffered future tiles to pop");
        }

        let buffered_future_tiles = self.buffered_future_tiles_opt.as_mut().unwrap();

        let (next_tick, next_x, next_y) = buffered_future_tiles.pop_front().unwrap();

        warn!("Prediction({:?}), Processing Buffered Next Tile Position! Tick: {:?}, Current: ({:?}, {:?}), Next: ({:?}, {:?})", false, next_tick, tile_x, tile_y, next_x, next_y);

        if buffered_future_tiles.is_empty() {
            self.buffered_future_tiles_opt = None;
        }

        (next_tick, next_x, next_y)
    }
}

// does not add (ax, ay) or (bx, by) to the buffer
fn buffer_pathfind_tiles(
    ax: i16,
    ay: i16,
    bx: i16,
    by: i16,
    buffer: &mut VecDeque<(Tick, i16, i16)>,
) {
    info!(
        "Pathfinding from ({:?}, {:?}) to ({:?}, {:?})",
        ax, ay, bx, by
    );

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

fn is_valid_tile_transition(ax: i16, ay: i16, bx: i16, by: i16) -> bool {
    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    let d_dis = dx + dy;
    if d_dis == 0 || d_dis > 2 || dx > 1 || dy > 1 {
        warn!(
            "from_tile and to_tile are not adjacent. ({:?}, {:?}) -> ({:?}, {:?}). Prediction: {:?}",
            ax, ay, bx, by, false,
        );
        return false;
    } else {
        return true;
    }
}
