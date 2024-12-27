use std::collections::VecDeque;

use bevy_ecs::component::Component;

use game_app_network::naia::{sequence_greater_than, Tick};

#[derive(Component)]
pub struct TickSkipper {
    // front of the queue is newest tick
    // back of the queue is oldest tick
    skipped_ticks: VecDeque<Tick>,
}

impl TickSkipper {
    pub fn new() -> Self {
        Self {
            skipped_ticks: VecDeque::new(),
        }
    }

    pub fn queue_skipped_tick(&mut self, tick: Tick) {
        if let Some(front_tick) = self.skipped_ticks.front() {
            if tick == *front_tick {
                return;
            }
            if sequence_greater_than(*front_tick, tick) {
                panic!(
                    "TickSkipper::queue_skipped_tick() - received out of order tick: {:?}",
                    tick
                );
            }
        }
        self.skipped_ticks.push_front(tick);
    }

    pub fn use_skipped_tick(&mut self, tick: Tick) -> bool {
        while let Some(back_tick) = self.skipped_ticks.back() {
            let back_tick = *back_tick;
            if sequence_greater_than(tick, back_tick) {
                self.skipped_ticks.pop_back();
            } else {
                break;
            }
        }

        if let Some(back_tick) = self.skipped_ticks.back() {
            if tick == *back_tick {
                self.skipped_ticks.pop_back();
                return true;
            } else if sequence_greater_than(*back_tick, tick) {
                // already received an update sooner than this tick, so skip it
                return true;
            }
        }

        return false;
    }
}
