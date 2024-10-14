use std::collections::{HashMap, VecDeque};

use naia_bevy_shared::{sequence_greater_than, Tick};

use logging::info;

use crate::{types::Direction, messages::PlayerCommand, resources::PlayerCommandEvent};

#[derive(Clone, Eq, PartialEq, Debug)]
struct ActionRecord {
    will_move: bool,
    buffered_movement: (i32, i32),
    pressed_map: HashMap<PlayerCommand, u64>,
}

impl ActionRecord {
   fn new() -> Self {
        Self {
            will_move: false,
            buffered_movement: (0, 0),
            pressed_map: HashMap::new(),
        }
    }
}

pub struct ActionManager {
    current_tick_opt: Option<Tick>,
    current_record: ActionRecord,

    // front is the oldest
    // back is the newest
    history: VecDeque<(Tick, ActionRecord)>,
}

impl ActionManager {
    pub fn new() -> Self {
        Self {
            current_tick_opt: None,
            current_record: ActionRecord::new(),

            history: VecDeque::new(),
        }
    }

    pub fn recv_rollback(&mut self, tick: Tick) {

        let tick = tick.wrapping_sub(1);

        info!("ActionManager Rollback -> Tick({:?})", tick);

        while let Some((history_tick, _)) = self.history.front() {
            let history_tick = *history_tick;
            if sequence_greater_than(tick, history_tick) {
                self.history.pop_front();
                info!("Popped FrontTick({:?})", history_tick);
            } else {
                if history_tick != tick {
                    panic!("ActionManager::recv_rollback called with a Tick({:?}) that doesn't match the FrontTick({:?}.", tick, history_tick);
                } else {
                    break;
                }
            }
        }

        let (_, record) = self.history.pop_front().unwrap();

        info!("Setting CurrentTick to {:?}", tick);
        self.current_tick_opt = Some(tick);
        self.current_record = record.clone();
        self.history.clear();
    }

    pub fn recv_command_events(&mut self, tick: Tick, command_events: Vec<PlayerCommandEvent>) {

        info!("PlayerCommandEvents -> ActionManager for Tick({:?})", tick);

        if let Some(current_tick) = self.current_tick_opt {
            if tick != current_tick.wrapping_add(1) {
                panic!("ActionManager::recv_command_events called with a Tick({:?}) that doesn't match the NextTick({:?}).", tick, current_tick.wrapping_add(1));
            }

            self.history.push_back((current_tick, self.current_record.clone()));
        }

        self.current_tick_opt = Some(tick);

        let mut dx = 0;
        let mut dy = 0;

        for command_event in command_events {

            match command_event {
                PlayerCommandEvent::Pressed(command, duration_ms) => {
                    match command {
                        PlayerCommand::Up => dy = -1,
                        PlayerCommand::Down => dy = 1,
                        PlayerCommand::Left => dx = -1,
                        PlayerCommand::Right => dx = 1,
                    }
                    if !self.current_record.pressed_map.contains_key(&command) {
                        self.current_record.pressed_map.insert(command, duration_ms as u64);
                    } else {
                        let prev_duration_ms = self.current_record.pressed_map.get_mut(&command).unwrap();
                        *prev_duration_ms += duration_ms as u64;
                    }
                    let total_duration = self.current_record.pressed_map.get(&command).unwrap();
                    if *total_duration >= 150 {
                        self.current_record.will_move = true;
                    }
                }
                PlayerCommandEvent::Released(command, _duration_ms) => {
                    match command {
                        PlayerCommand::Up => if dy == -1 { dy = 0 },
                        PlayerCommand::Down => if dy == 1 { dy = 0 },
                        PlayerCommand::Left => if dx == -1 { dx = 0 },
                        PlayerCommand::Right => if dx == 1 { dx = 0 },
                    }
                    self.current_record.pressed_map.remove(&command);
                }
            }
        }

        if dx != 0 {
            self.current_record.buffered_movement.0 = dx;
        }
        if dy != 0 {
            self.current_record.buffered_movement.1 = dy;
        }
    }

    pub fn take_movement(&mut self, tick: Tick) -> Option<Direction> {
        let Some(current_tick) = self.current_tick_opt else {
            panic!("ActionManager::take_movement called without a CurrentTick.");
        };
        if tick != current_tick {
            panic!("ActionManager::take_movement called with a Tick({:?}) that doesn't match the CurrentTick({:?}.", tick, current_tick);
        }

        if self.current_record.will_move {
            self.current_record.will_move = false;
            let direction = Direction::from_coords(self.current_record.buffered_movement.0 as f32, self.current_record.buffered_movement.1 as f32);
            self.current_record.buffered_movement = (0, 0);
            Some(direction)
        } else {
            None
        }
    }
}