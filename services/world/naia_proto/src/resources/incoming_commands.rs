use std::collections::{HashMap, VecDeque};

use naia_bevy_shared::Tick;

use crate::messages::{PlayerCommands, PlayerCommandStream, PlayerCommand};

const TICK_DURATION_MS: u16 = 40; // TODO: move to config

// Intended to be encapsulated within a Client or Server specific Resource!
pub struct IncomingCommands {
    map: HashMap<PlayerCommand, IncomingCommandStream>,
}

impl IncomingCommands {
    pub fn new() -> Self {

        let mut map = HashMap::new();
        map.insert(PlayerCommand::Forward, IncomingCommandStream::new());
        map.insert(PlayerCommand::Backward, IncomingCommandStream::new());
        map.insert(PlayerCommand::Left, IncomingCommandStream::new());
        map.insert(PlayerCommand::Right, IncomingCommandStream::new());

        Self {
            map
        }
    }

    pub fn recv_incoming_command(&mut self, tick: Tick, player_commands_opt: Option<PlayerCommands>) {

        for (command, stream) in self.map.iter_mut() {
            let Some(player_commands) = player_commands_opt.as_ref() else {
                stream.recv_none(tick);
                continue;
            };
            let Some(command_stream) = player_commands.get(command) else {
                stream.recv_none(tick);
                continue;
            };

            stream.recv_stream(tick, command_stream);
        }
    }

    pub fn pop_incoming_commands(&mut self, tick: Tick) -> Vec<PlayerCommandEvent> {

        let mut output = Vec::new();

        for (command, stream) in self.map.iter_mut() {
            stream.pop_commands(command, tick, &mut output);
        }

        output
    }
}

// IncomingCommandStream

struct IncomingCommandStream {
    current_tick_opt: Option<Tick>,
    // VecDeque<(pressed, duration (ms), is_finished)>))
    durations: VecDeque<(bool, u16, bool)>,
}

impl IncomingCommandStream {
    fn new() -> Self {
        Self {
            current_tick_opt: None,
            durations: VecDeque::new()
        }
    }

    fn recv_stream(&mut self, tick: Tick, key_stream: &PlayerCommandStream) {
        self.current_tick_opt = Some(tick);

        let mut pressed = key_stream.start_pressed();
        let durations = key_stream.durations();
        let mut remaining_duration = TICK_DURATION_MS;

        if durations.is_empty() {

            if let Some((back_pressed, back_duration, back_finished)) = self.durations.back_mut() {
                if *back_pressed == pressed {
                    *back_duration = *back_duration + remaining_duration;
                    if *back_finished {
                        panic!("obviously not finished!");
                    }
                } else {

                    if *back_finished {
                        panic!("obviously not finished!");
                    }
                    *back_finished = true;
                    self.durations.push_back((pressed, remaining_duration, false));
                }
            } else {
                self.durations.push_back((pressed, remaining_duration, false));
            }

        } else {
            for (index, duration) in durations.iter().enumerate() {

                let duration_ms: u16 = duration.to();

                if duration_ms <= remaining_duration {
                    remaining_duration -= duration_ms;
                } else {
                    remaining_duration = 0;
                }

                if index == 0 && !self.durations.is_empty() {
                    let (back_pressed, back_duration, back_finished) = self.durations.back_mut().unwrap();

                    if *back_finished {
                        panic!("obviously not finished!");
                    }
                    *back_finished = true;

                    if *back_pressed == pressed {
                        *back_duration = *back_duration + duration_ms;
                    } else {
                        self.durations.push_back((pressed, duration_ms, true));
                    }

                    pressed = !pressed;
                    continue;
                }

                self.durations.push_back((pressed, duration_ms, true));
                pressed = !pressed;
            }

            self.durations.push_back((pressed, remaining_duration, false));
        }
    }

    fn recv_none(&mut self, tick: Tick) {
        self.current_tick_opt = Some(tick);

        if let Some((back_pressed, back_duration, back_finished)) = self.durations.back_mut() {
            if *back_pressed {
                *back_finished = true;
            } else {
                if *back_finished {
                    panic!("obviously not finished!");
                }

                if u16::MAX - *back_duration < TICK_DURATION_MS {
                    *back_duration = u16::MAX;
                } else {
                    *back_duration = *back_duration + TICK_DURATION_MS;
                }
                return;
            }
        }

        self.durations.push_back((false, TICK_DURATION_MS, false));
    }

    pub(crate) fn pop_commands(&mut self, command: &PlayerCommand, tick: Tick, output: &mut Vec<PlayerCommandEvent>) {
        if self.current_tick_opt != Some(tick) {
            panic!("pop_commands called with incorrect tick! current_tick_opt: {:?}, tick: {:?}", self.current_tick_opt, tick);
        }

        let mut last_pressed = false;

        while self.durations.len() > 1 { // leave the last one for the next tick
            let (pressed, duration, finished) = self.durations.pop_front().unwrap();
            if !finished {
                panic!("obviously not finished!");
            }

            if pressed {
                output.push(PlayerCommandEvent::Pressed(*command, duration));
                last_pressed = true;
            } else {
                output.push(PlayerCommandEvent::Released(*command));
                last_pressed = false;
            }
        }

        if let Some((pressed, duration, finished)) = self.durations.front() {
            if *finished {
                panic!("last one should not be finished!");
            }

            if *pressed {
                output.push(PlayerCommandEvent::Held(*command, *duration));
            } else {
                if last_pressed {
                    output.push(PlayerCommandEvent::Released(*command));
                }
            }
        }
    }
}

// KeyEvent
pub enum PlayerCommandEvent {
    // key, duration (ms)
    Pressed(PlayerCommand, u16),
    // key, duration (ms)
    Held(PlayerCommand, u16),
    // key
    Released(PlayerCommand),
}

mod tests {
    use super::*;

    #[test]
    fn test_1() {

        let key_stream = PlayerCommandStream::new(true);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);

        assert_eq!(command_stream.durations.len(), 1);
        assert_eq!(command_stream.durations[0], (true, 40, false));
    }

    #[test]
    fn test_2() {

        let key_stream = PlayerCommandStream::new(false);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);

        assert_eq!(command_stream.durations.len(), 1);
        assert_eq!(command_stream.durations[0], (false, 40, false));
    }

    #[test]
    fn test_3() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);

        assert_eq!(command_stream.durations.len(), 2);
        assert_eq!(command_stream.durations, [(true, 28, true), (false, TICK_DURATION_MS - 28, false)]);
    }

    #[test]
    fn test_4() {

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_none(0);

        assert_eq!(command_stream.durations.len(), 1);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS, false)]);
    }

    #[test]
    fn test_5() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        assert_eq!(command_stream.durations.len(), 2);
        assert_eq!(command_stream.durations, [(true, 28, true), (false, (TICK_DURATION_MS + TICK_DURATION_MS - 28), false)]);
    }

    #[test]
    fn test_6() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);


        assert_eq!(command_stream.durations.len(), 3);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS, true), (true, 28, true), (false, TICK_DURATION_MS - 28, false)]);
    }

    #[test]
    fn test_7() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);


        assert_eq!(command_stream.durations.len(), 2);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS + 28, true), (true, TICK_DURATION_MS - 28, false)]);
    }

    #[test]
    fn test_8() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        assert_eq!(command_stream.durations.len(), 3);
        assert_eq!(command_stream.durations, [(false, 28, true), (true, TICK_DURATION_MS - 28, true), (false, TICK_DURATION_MS, false)]);
    }

    #[test]
    fn test_9() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        assert_eq!(command_stream.durations.len(), 4);
        assert_eq!(command_stream.durations, [(true, 21, true), (false, 13, true), (true, 5, true), (false, (TICK_DURATION_MS + TICK_DURATION_MS - 21 - 13 - 5), false)]);
    }

    #[test]
    fn test_10() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);

        assert_eq!(command_stream.durations.len(), 5);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS, true), (true, 21, true), (false, 13, true), (true, 5, true), (false, TICK_DURATION_MS - 21 - 13 - 5, false)]);
    }

    #[test]
    fn test_11() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);

        assert_eq!(command_stream.durations.len(), 4);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS + 21, true), (true, 13, true), (false, 5, true), (true, TICK_DURATION_MS - 21 - 13 - 5, false)]);
    }

    #[test]
    fn test_12() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        assert_eq!(command_stream.durations.len(), 5);
        assert_eq!(command_stream.durations, [(false, 21, true), (true, 13, true), (false, 5, true), (true, TICK_DURATION_MS - 21 - 13 - 5, true), (false, TICK_DURATION_MS, false)]);
    }

    #[test]
    fn test_13() {

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_none(0);
        command_stream.recv_none(1);
        command_stream.recv_none(2);

        assert_eq!(command_stream.durations.len(), 1);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS * 3, false)]);
    }

    #[test]
    fn test_14() {

        let mut key_stream = PlayerCommandStream::new(false);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_stream(1, &key_stream);
        command_stream.recv_stream(2, &key_stream);

        assert_eq!(command_stream.durations.len(), 1);
        assert_eq!(command_stream.durations, [(false, TICK_DURATION_MS * 3, false)]);
    }

    #[test]
    fn test_15() {

        let mut key_stream = PlayerCommandStream::new(true);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_stream(1, &key_stream);
        command_stream.recv_stream(2, &key_stream);

        assert_eq!(command_stream.durations.len(), 1);
        assert_eq!(command_stream.durations, [(true, TICK_DURATION_MS * 3, false)]);
    }

    #[test]
    fn test_16() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new();
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_stream(1, &key_stream);
        command_stream.recv_stream(2, &key_stream);

        assert_eq!(command_stream.durations.len(), 7);
        assert_eq!(command_stream.durations, [
            (true, 21, true),
            (false, 5, true),
            (true, TICK_DURATION_MS - 21 - 5 + 21, true),
            (false, 5, true),
            (true, TICK_DURATION_MS - 21 - 5 + 21, true),
            (false, 5, true),
            (true, TICK_DURATION_MS - 21 - 5, false)
        ]);
    }
}