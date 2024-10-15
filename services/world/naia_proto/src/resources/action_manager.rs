use std::collections::{HashMap, VecDeque};

use naia_bevy_shared::{sequence_greater_than, Tick};

use logging::info;

use crate::{types::Direction, messages::PlayerCommand, resources::PlayerCommandEvent};

#[derive(Clone, Eq, PartialEq, Debug)]
struct ActionRecord {
    //will_move: bool,
    buffered_movement: (i32, i32),
    pressed_map: HashMap<PlayerCommand, u64>,
}

impl ActionRecord {
   fn new() -> Self {
        Self {
            //will_move: false,
            buffered_movement: (0, 0),
            pressed_map: HashMap::new(),
        }
    }

    pub(crate) fn recv_press(&mut self, command: PlayerCommand, duration_ms: u8) {
        let duration_ms = duration_ms as i32;
        match command {
            PlayerCommand::Up => {
                self.buffered_movement.1 = (self.buffered_movement.1.min(0) - duration_ms).max(-160);
            },
            PlayerCommand::Down => {
                self.buffered_movement.1 = (self.buffered_movement.1.max(0) + duration_ms).min(160);
            },
            PlayerCommand::Left => {
                self.buffered_movement.0 = (self.buffered_movement.0.min(0) - duration_ms).max(-160);
            },
            PlayerCommand::Right => {
                self.buffered_movement.0 = (self.buffered_movement.0.max(0) + duration_ms).min(160);
            },
        }
    }

    pub(crate) fn recv_release(&mut self, command: PlayerCommand, duration_ms: u8) {
        let duration_ms = duration_ms as i32;
        let duration_ms = duration_ms;

        match command {
            PlayerCommand::Up => if self.buffered_movement.1 < 0 {
                self.buffered_movement.1 = (self.buffered_movement.1 + duration_ms).min(0);
            },
            PlayerCommand::Down => if self.buffered_movement.1 > 0 {
                self.buffered_movement.1 = (self.buffered_movement.1 - duration_ms).max(0);
            },
            PlayerCommand::Left => if self.buffered_movement.0 < 0 {
                self.buffered_movement.0 = (self.buffered_movement.0 + duration_ms).min(0);
            },
            PlayerCommand::Right => if self.buffered_movement.0 > 0 {
                self.buffered_movement.0 = (self.buffered_movement.0 - duration_ms).max(0);
            },
        }
    }

    pub(crate) fn take_movement(&mut self) -> Option<Direction> {
        let mut dx = self.buffered_movement.0;
        let mut dy = self.buffered_movement.1;
        if dx.abs() < 150 && dy.abs() < 150 {
            return None;
        }

        let mut rx = 0;
        let mut ry = 0;

        let dx_threshold = if dx.abs() > dy.abs() { 150 } else { 1 };
        let dy_threshold = if dy.abs() > dx.abs() { 150 } else { 1 };

        if dx > dx_threshold {
            rx = 1;
        } else if dx < -dx_threshold {
            rx = -1;
        }
        if dy > dy_threshold {
            ry = 1;
        } else if dy < -dy_threshold {
            ry = -1;
        }

        self.buffered_movement = (0, 0);
        return Some(Direction::from_coords(rx as f32, ry as f32));
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

        //info!("ActionManager Rollback -> Tick({:?})", tick);

        while let Some((history_tick, _)) = self.history.front() {
            let history_tick = *history_tick;
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

        let (_, record) = self.history.pop_front().unwrap();

        //info!("Setting CurrentTick to {:?}", tick);
        self.current_tick_opt = Some(tick);
        self.current_record = record.clone();
        self.history.clear();
    }

    pub fn recv_command_events(&mut self, tick: Tick, command_events: Vec<PlayerCommandEvent>) {

        //info!("PlayerCommandEvents -> ActionManager for Tick({:?})", tick);

        if let Some(current_tick) = self.current_tick_opt {
            if tick != current_tick.wrapping_add(1) {
                panic!("ActionManager::recv_command_events called with a Tick({:?}) that doesn't match the NextTick({:?}).", tick, current_tick.wrapping_add(1));
            }

            self.history.push_back((current_tick, self.current_record.clone()));
        }

        self.current_tick_opt = Some(tick);

        for command_event in command_events {

            match command_event {
                PlayerCommandEvent::Pressed(command, duration_ms) => {
                    if !self.current_record.pressed_map.contains_key(&command) {
                        self.current_record.pressed_map.insert(command, duration_ms as u64);
                    } else {
                        let prev_duration_ms = self.current_record.pressed_map.get_mut(&command).unwrap();
                        *prev_duration_ms += duration_ms as u64;
                    }
                    self.current_record.recv_press(command, duration_ms);
                }
                PlayerCommandEvent::Released(command, duration_ms) => {
                    self.current_record.pressed_map.remove(&command);

                    self.current_record.recv_release(command, duration_ms);
                }
            }
        }
    }

    pub fn take_movement(&mut self, tick: Tick) -> Option<Direction> {
        let Some(current_tick) = self.current_tick_opt else {
            panic!("ActionManager::take_movement called without a CurrentTick.");
        };
        if tick != current_tick {
            panic!("ActionManager::take_movement called with a Tick({:?}) that doesn't match the CurrentTick({:?}.", tick, current_tick);
        }

        self.current_record.take_movement()
    }
}