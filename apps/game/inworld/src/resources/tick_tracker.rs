use std::default::Default;

use bevy_ecs::prelude::Resource;

use game_app_network::naia::{sequence_greater_than, Tick};

#[derive(Resource)]
pub struct TickTracker {
    last_processed_server_tick_opt: Option<Tick>,
}

impl Default for TickTracker {
    fn default() -> Self {
        Self { last_processed_server_tick_opt: None }
    }
}

impl TickTracker {
    pub fn set_last_processed_server_tick(&mut self, tick: Tick) {
        if let Some(existing_tick) = self.last_processed_server_tick_opt {
            if sequence_greater_than(tick, existing_tick) {
                self.last_processed_server_tick_opt = Some(tick);
            }
        } else {
            self.last_processed_server_tick_opt = Some(tick);
        }
    }

    pub fn last_processed_server_tick(&self) -> Option<Tick> {
        self.last_processed_server_tick_opt
    }
}