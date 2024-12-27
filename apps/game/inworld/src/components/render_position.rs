use std::collections::VecDeque;

use bevy_ecs::component::Component;

use game_engine::{
    math::Vec2,
    time::Instant,
};
use game_app_network::{naia::{sequence_less_than, GameInstant, Tick}, world::{components::NextTilePosition, constants::TILE_SIZE, WorldClient}};

#[derive(Component, Clone)]
pub struct RenderPosition {
    // front is oldest, back is newest
    position_queue: VecDeque<(Vec2, Tick)>,
    last_render_instant: Instant,
    interp_instant: GameInstant,
}

impl RenderPosition {
    pub fn new(
        next_tile_position: &NextTilePosition,
        tick: Tick,
        tick_instant: GameInstant,
    ) -> Self {
        let x = next_tile_position.x() as f32 * TILE_SIZE;
        let y = next_tile_position.y() as f32 * TILE_SIZE;

        let mut me = Self {
            position_queue: VecDeque::new(),
            last_render_instant: Instant::now(),
            interp_instant: tick_instant,
        };

        me.position_queue.push_back((Vec2::new(x, y), tick));

        me
    }

    pub fn recv_position(&mut self, position: Vec2, tick: Tick) {

        let mut back_tick_opt = None;

        // make sure ticks are in order
        loop {
            if let Some((_, back_tick)) = self.position_queue.back() {
                if sequence_less_than(tick, *back_tick) || tick == *back_tick {
                    // warn!("recv_position() - received out of order tick: {:?}", tick);
                    self.position_queue.pop_back();
                } else {
                    back_tick_opt = Some(*back_tick);
                    break;
                }
            } else {
                break;
            }
        }

        self.position_queue.push_back((position, tick));

        // let host = if is_server { "Server" } else { "Client" };
        // let rollback = if is_rollback { "Rollback" } else { "" };
        // info!(
        //     "{:?}({:?}), Tick: {:?}, Pos: ({:?}, {:?})",
        //     host, rollback, tick, position.x, position.y
        // );
    }

    pub fn recv_rollback(&mut self, server_render_pos: &RenderPosition) {
        //info!("recv_rollback()");

        self.position_queue = server_render_pos.position_queue.clone();
    }

    // returns (position, velocity)
    pub fn render(&mut self, client: &WorldClient, now: &Instant) -> (Vec2, Vec2) {
        {
            let duration_elapsed = self.last_render_instant.elapsed(&now);
            let duration_ms = duration_elapsed.as_secs_f32() * 1000.0;

            let adjust: f32 = match self.position_queue.len() {
                0 => 0.7,
                1 => 0.8,
                2 => 0.9,
                3 => 1.0,
                4 => 1.1,
                5 => 1.2,
                _ => 1.3,
            };
            let duration_ms = duration_ms * adjust;

            self.advance_millis(client, duration_ms as u32);
        }

        //info!("duration_ms: {:?}", duration_ms);
        self.last_render_instant = now.clone();

        if self.position_queue.len() < 2 {
            if self.position_queue.len() < 1 {
                panic!("queue is empty");
            }

            let (position, _) = self.position_queue.get(0).unwrap();
            return (*position, Vec2::ZERO);
        }

        let (prev_pos, prev_instant, next_pos, next_instant) = {
            let (prev_pos, prev_tick) = self.position_queue.get(0).unwrap();
            let (next_pos, next_tick) = self.position_queue.get(1).unwrap();

            let prev_instant = client
                .tick_to_instant(*prev_tick)
                .expect("client not initialized?");
            let next_instant = client
                .tick_to_instant(*next_tick)
                .expect("client not initialized?");

            (
                *prev_pos,
                prev_instant,
                *next_pos,
                next_instant,
            )
        };

        let prev_to_interp = prev_instant.offset_from(&self.interp_instant) as f32;
        if prev_to_interp < 0.0 {
            return (prev_pos, Vec2::ZERO);
        }
        let interp_to_next = self.interp_instant.offset_from(&next_instant) as f32;
        let total = prev_to_interp + interp_to_next;
        let interp = prev_to_interp / total;

        let interp_x = prev_pos.x + ((next_pos.x - prev_pos.x) * interp);
        let interp_y = prev_pos.y + ((next_pos.y - prev_pos.y) * interp);

        let interp_pos = Vec2::new(interp_x, interp_y);
        (interp_pos, next_pos - prev_pos)
    }

    pub fn advance_millis(&mut self, client: &WorldClient, millis: u32) {
        self.interp_instant = self.interp_instant.add_millis(millis);

        loop {
            if self.position_queue.len() < 1 {
                panic!("queue is empty");
            }

            // // This makes it so interp_instance always is in between the points in the queue
            // let (_, _, prev_tick) = self.queue.get(0).unwrap();
            // let prev_instant = client
            //     .tick_to_instant(*prev_tick)
            //     .expect("client not initialized?");

            // if prev_instant.is_more_than(&self.interp_instant) {
            //     self.interp_instant = prev_instant.clone();
            //     break;
            // }

            if self.position_queue.len() < 2 {
                break;
            }

            let (_, next_tick) = self.position_queue.get(1).unwrap();
            let next_instant = client
                .tick_to_instant(*next_tick)
                .expect("client not initialized?");
            if self.interp_instant.is_more_than(&next_instant) {
                self.position_queue.pop_front();
            } else {
                break;
            }
        }

        // {
        //     // this pops any future positions that are the same as the current position (no interpolation needed)
        //     if self.queue.len() >= 3 {
        //         if eventually_differs(&self.queue) {
        //             let (front_x, front_y, _) = self.queue.front().unwrap();
        //             let front_x = *front_x;
        //             let front_y = *front_y;
        //             while let Some((x, y, tick)) = self.queue.get(1) {
        //                 let x = *x;
        //                 let y = *y;
        //                 let tick = *tick;
        //
        //                 if x == front_x && y == front_y {
        //                     self.queue.pop_front();
        //                     self.interp_instant = client
        //                         .tick_to_instant(tick)
        //                         .expect("client not initialized?");
        //                 } else {
        //                     break;
        //                 }
        //             }
        //         }
        //     }
        // }
    }
}

fn eventually_differs(queue: &VecDeque<(f32, f32, Tick)>) -> bool {
    let (front_x, front_y, _) = queue.front().unwrap();
    let front_x = *front_x;
    let front_y = *front_y;
    let mut index = 1;
    while let Some((x, y, _)) = queue.get(index) {
        if *x != front_x || *y != front_y {
            return true;
        }
        index += 1;
    }
    return false;
}
