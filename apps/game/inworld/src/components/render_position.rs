use std::collections::VecDeque;

use bevy_ecs::component::Component;

use game_engine::{world::{constants::TILE_SIZE, components::NextTilePosition}, time::Instant};

#[derive(Component, Clone)]
pub struct RenderPosition {
    queue: VecDeque<(f32, f32)>,
    interp: (f32, f32),
    last_render_instant: Instant,
}

impl RenderPosition {
    pub fn new(next_tile_position: &NextTilePosition) -> Self {
        let mut me = Self {
            queue: VecDeque::new(),
            interp: (next_tile_position.x() as f32 * TILE_SIZE, next_tile_position.y() as f32 * TILE_SIZE),
            last_render_instant: Instant::now(),
        };

        me.queue.push_back((next_tile_position.x() as f32 * TILE_SIZE, next_tile_position.y() as f32 * TILE_SIZE));

        me
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn queue_ref(&self) -> &VecDeque<(f32, f32)> {
        &self.queue
    }

    pub fn recv_position(&mut self, position: (f32, f32)) {
        self.queue.push_back(position);
    }

    pub fn recv_rollback(&mut self, _server_render_pos: &RenderPosition) {
        // TODO: implement?
    }

    pub fn render(&mut self, now: &Instant) -> (f32, f32) {
        let duration_ms = now.elapsed(&self.last_render_instant).as_secs_f32() * 1000.0;
        self.last_render_instant = now.clone();

        todo!()
    }
}
