use std::collections::VecDeque;

use bevy_ecs::component::Component;

use game_engine::{world::{constants::TILE_SIZE, components::NextTilePosition}, time::Instant, naia::GameInstant};
use game_engine::logging::warn;

#[derive(Component, Clone)]
pub struct RenderPosition {
    queue: VecDeque<(f32, f32, GameInstant)>,
    last_render_instant: Instant,
    interp_instant: GameInstant,
}

impl RenderPosition {
    pub fn new(next_tile_position: &NextTilePosition, instant: GameInstant) -> Self {

        let x = next_tile_position.x() as f32 * TILE_SIZE;
        let y = next_tile_position.y() as f32 * TILE_SIZE;

        let mut me = Self {
            queue: VecDeque::new(),
            last_render_instant: Instant::now(),
            interp_instant: instant.clone(),
        };

        me.queue.push_back((x, y, instant));

        me
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn queue_ref(&self) -> &VecDeque<(f32, f32, GameInstant)> {
        &self.queue
    }

    pub fn recv_position(&mut self, position: (f32, f32), instant: GameInstant) {
        self.queue.push_back((position.0, position.1, instant));
    }

    pub fn recv_rollback(&mut self, _server_render_pos: &RenderPosition) {
        // TODO: implement?
    }

    pub fn render(&mut self, now: &Instant) -> (f32, f32) {

        let queue_len = self.queue.len();

        {
            let duration_elapsed = self.last_render_instant.elapsed(&now);
            let duration_ms = (duration_elapsed.as_secs_f32() * 1000.0);

            let adjust: f32 = match queue_len {
                2 => 0.9,
                3 => 1.0,
                4 => 1.1,
                5 => 1.2,
                _ => 1.0,
            };
            let duration_ms = duration_ms * adjust;

            self.interp_instant = self.interp_instant.add_millis(duration_ms as u32);
        }

        //info!("duration_ms: {:?}", duration_ms);
        self.last_render_instant = now.clone();

        // can't move to next position in queue unless there are two positions in the queue
        if queue_len < 2 {
            if queue_len < 1 {
                panic!("queue is empty");
            }

            warn!("queue is too short, returning current position");
            let (target_x, target_y, _) = self.queue.get(0).unwrap();
            return (*target_x, *target_y);
        }

        // if queue is too long...
        if queue_len > 4 {
            warn!("queue is too long, truncating");

            while queue_len > 3 {
                self.queue.pop_front();
            }
        }

        let (prev_x, prev_y, prev_instant) = self.queue.get(0).unwrap();
        let prev_x = *prev_x;
        let prev_y = *prev_y;

        let (next_x, next_y, next_instant) = self.queue.get(1).unwrap();
        let next_x = *next_x;
        let next_y = *next_y;

        if prev_instant.is_more_than(&self.interp_instant) {
            self.interp_instant = prev_instant.clone();
        }

        if self.interp_instant.is_more_than(&next_instant) {
            // TODO.. extrapolate?
            self.queue.pop_front();
            return (next_x, next_y);
        }

        let prev_to_interp = prev_instant.offset_from(&self.interp_instant) as f32;
        let interp_to_next = self.interp_instant.offset_from(&next_instant) as f32;
        let total = prev_to_interp + interp_to_next;
        let interp = prev_to_interp / total;

        let interp_x = prev_x + ((next_x - prev_x) * interp);
        let interp_y = prev_y + ((next_y - prev_y) * interp);

        (interp_x, interp_y)
    }
}
