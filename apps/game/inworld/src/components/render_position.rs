use std::collections::VecDeque;

use bevy_ecs::component::Component;

use game_engine::{world::{constants::TILE_SIZE, components::NextTilePosition}, time::Instant};

#[derive(Component, Clone)]
pub struct RenderPosition {
    queue: VecDeque<(f32, f32)>,
    position: (f32, f32),
    last_render_instant: Instant,
}

impl RenderPosition {
    pub fn new(next_tile_position: &NextTilePosition) -> Self {

        let x = next_tile_position.x() as f32 * TILE_SIZE;
        let y = next_tile_position.y() as f32 * TILE_SIZE;

        let mut me = Self {
            queue: VecDeque::new(),
            position: (x, y),
            last_render_instant: Instant::now(),
        };

        me.queue.push_back((x, y));

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
        let duration_elapsed = self.last_render_instant.elapsed(&now);
        let duration_ms = (duration_elapsed.as_secs_f32() * 1000.0);
        //info!("duration_ms: {:?}", duration_ms);
        self.last_render_instant = now.clone();

        // can't move to next position in queue unless there are two positions in the queue
        if self.queue.len() < 2 {
            return self.position;
        }

        let (target_x, target_y) = self.queue.front().unwrap();
        let target_x = *target_x;
        let target_y = *target_y;

        // calculate the distance to the target position
        let dx = target_x - self.position.0;
        let dy = target_y - self.position.1;
        let distance_to_target = (dx * dx + dy * dy).sqrt();

        const INTERP_SPEED: f32 = 0.2;

        // calculate the distance to move this frame

        //info!("duration_ms: {:?}", duration_ms);
        let move_distance = INTERP_SPEED * (duration_ms);

        if move_distance == 0.0 {
            //warn!("move_distance is zero? speed * duration_ms = distance: {:?} * {:?} = {:?}", INTERP_SPEED, duration_ms, move_distance);
            return self.position;
        }

        // if the distance to move is greater than the distance to the target position
        // then we have reached the target position
        if move_distance >= distance_to_target {
            self.position = (target_x, target_y);
            self.queue.pop_front();
        } else {
            // move towards the target position
            let angle = dy.atan2(dx);
            let new_x = self.position.0 + move_distance * angle.cos();
            let new_y = self.position.1 + move_distance * angle.sin();
            self.position = (new_x, new_y);
        }

        return self.position;
    }
}
