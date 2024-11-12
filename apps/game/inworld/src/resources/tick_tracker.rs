use std::default::Default;

use bevy_ecs::prelude::Resource;

use game_engine::naia::Tick;

#[derive(Resource)]
pub struct TickTracker {
    last_processed_server_tick: Option<Tick>,
}

impl Default for TickTracker {
    fn default() -> Self {
        Self {
            last_processed_server_tick: None,
        }
    }
}

impl TickTracker {
    pub fn set_last_processed_server_tick(&mut self, tick: Tick) {
        self.last_processed_server_tick = Some(tick);
    }

    pub fn last_processed_server_tick(&self) -> Tick {
        self.last_processed_server_tick.unwrap()
    }
}
