use std::collections::VecDeque;

use naia_bevy_shared::{sequence_greater_than, Tick};

use logging::info;

use crate::{types::Direction, resources::CommandTimeline};

#[derive(Clone, Eq, PartialEq, Debug)]
struct ActionRecord {
    tick_opt: Option<Tick>,
    buffered_movement: Option<Direction>,
}

impl ActionRecord {
   fn new() -> Self {
        Self {
            tick_opt: None,
            buffered_movement: None,
        }
   }

   fn tick(&self) -> Tick {
       self.tick_opt.unwrap()
   }

   fn tick_opt(&self) -> Option<Tick> {
       self.tick_opt
   }

   fn set_tick(&mut self, tick: Tick) {
       self.tick_opt = Some(tick);
   }

   fn recv_command_timeline(&mut self, tick: Tick, command: CommandTimeline) {
       self.tick_opt = Some(tick);

       let (dx, dy) = self.buffered_movement.as_ref().map(|direction| direction.to_delta()).unwrap_or((0, 0));
       let (dx, dy) = command.get_movement_vector(dx, dy);
       if dx == 0 && dy == 0 {
           self.buffered_movement = None;
       } else {
           self.buffered_movement = Direction::from_delta(dx, dy);
       }
   }

    fn take_movement(&mut self) -> Option<Direction> {
        self.buffered_movement.take()
   }
}

pub struct ActionManager {
    current_record: ActionRecord,

    // front is the oldest
    // back is the newest
    history: VecDeque<ActionRecord>,
}

impl ActionManager {
    pub fn new() -> Self {
        Self {
            current_record: ActionRecord::new(),

            history: VecDeque::new(),
        }
    }

    pub fn recv_rollback(&mut self, tick: Tick) {

        let tick = tick.wrapping_sub(1);

        info!("ActionManager Rollback -> Tick({:?})", tick);

        while let Some(record) = self.history.front() {
            let history_tick = record.tick();
            if sequence_greater_than(tick, history_tick) {
                self.history.pop_front();
                //info!("Popped FrontTick({:?})", history_tick);
            } else {
                if history_tick != tick {
                    panic!("ActionManager::recv_rollback called with a Tick({:?}) that doesn't match the FrontTick({:?}.", tick, history_tick);
                } else {
                    break;
                }
            }
        }

        let record = self.history.pop_front().unwrap();
        // clear history, will rebuild during command playback
        self.history.clear();

        self.current_record = record.clone();
        if self.current_record.tick() != tick {
            panic!("Current Record Tick({:?}) doesn't match the Rollback Tick({:?}).", self.current_record.tick(), tick);
        }


        // this is KEY, and it's not really about rollback ...
        // rollback happens right now, whenever a NextTilePosition is received
        // when that happens, we don't want to buffer anything
        // and this helps to eventually converge on a synchronized state
        //
        // if we didn't do this right here, we'd need to transmit
        // the buffered movement from server to client every tick! to keep them in sync!
        //
        // it's possible that we should be clearing this at the same time
        // as we call `tile_movement.recv_updated_next_tile_position`...
        //
        info!("Resetting Buffered Movement");
        self.current_record.buffered_movement = None;
    }

    pub fn recv_command_timeline(&mut self, tick: Tick, command_timeline: CommandTimeline) {

        //info!("PlayerCommandEvents -> ActionManager for Tick({:?})", tick);

        // buffer current record
        if let Some(current_tick) = self.current_record.tick_opt() {
            if tick != current_tick.wrapping_add(1) {
                panic!("ActionManager::recv_command_events called with a Tick({:?}) that doesn't match the NextTick({:?}).", tick, current_tick.wrapping_add(1));
            }

            self.history.push_back(self.current_record.clone());
        }

        self.current_record.recv_command_timeline(tick, command_timeline);
    }

    pub fn take_movement(&mut self, tick: Tick) -> Option<Direction> {
        let Some(current_tick) = self.current_record.tick_opt else {
            panic!("ActionManager::take_movement called without a CurrentTick.");
        };
        if tick != current_tick {
            panic!("ActionManager::take_movement called with a Tick({:?}) that doesn't match the CurrentTick({:?}.", tick, current_tick);
        }

        self.current_record.take_movement()
    }
}