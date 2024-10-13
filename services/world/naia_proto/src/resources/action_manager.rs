
use naia_bevy_shared::Tick;

use crate::{types::Direction, messages::PlayerCommand, resources::PlayerCommandEvent};

pub struct ActionManager {
    buffered_movement: (i32, i32),
    will_move: bool,
    last_tick: Tick,
}

impl ActionManager {
    pub fn new() -> Self {
        Self {
            buffered_movement: (0, 0),
            will_move: false,
            last_tick: 0,
        }
    }

    pub fn recv_rollback(&mut self) {
        self.buffered_movement = (0, 0);
        self.will_move = false;
        self.last_tick = 0;
    }

    pub fn recv_command_events(&mut self, tick: Tick, command_events: Vec<PlayerCommandEvent>) {

        self.last_tick = tick;

        for command_event in command_events {

            match command_event {
                PlayerCommandEvent::Pressed(command, duration_ms) | PlayerCommandEvent::Held(command, duration_ms) => {
                    match command {
                        PlayerCommand::Up => self.buffered_movement.1 += -1 * duration_ms as i32,
                        PlayerCommand::Down => self.buffered_movement.1 += duration_ms as i32,
                        PlayerCommand::Left => self.buffered_movement.0 += -1 * duration_ms as i32,
                        PlayerCommand::Right => self.buffered_movement.0 += duration_ms as i32,
                    }
                    if duration_ms > 150 {
                        self.will_move = true;
                    }
                }
                PlayerCommandEvent::Released(command, duration_ms) => {
                    // match command {
                    //     PlayerCommand::Up => self.buffered_movement.1 += duration_ms as i32,
                    //     PlayerCommand::Down => self.buffered_movement.1 += -1 * duration_ms as i32,
                    //     PlayerCommand::Left => self.buffered_movement.0 += duration_ms as i32,
                    //     PlayerCommand::Right => self.buffered_movement.0 += -1 * duration_ms as i32,
                    // }
                }
            }
        }
    }

    pub fn take_movement(&mut self, tick: Tick) -> Option<Direction> {
        if tick != self.last_tick {
            panic!("ActionManager::take_movement called with a Tick({:?}) that doesn't match the LastTick({:?}.", tick, self.last_tick);
        }

        if self.will_move {
            self.will_move = false;
            let direction = Direction::from_coords(self.buffered_movement.0 as f32, self.buffered_movement.1 as f32);
            self.buffered_movement = (0, 0);
            Some(direction)
        } else {
            None
        }
    }
}