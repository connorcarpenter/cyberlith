use std::collections::VecDeque;

use naia_bevy_shared::Tick;
use input::Key;
use crate::messages::{CommandReadState, KeyCommand, KeyStream};

const TICK_DURATION_MS: u16 = 40; // TODO: move to config

// Intended to be encapsulated within a Client or Server specific Resource!
pub struct CommandManager {
    keys_state: KeysState,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            keys_state: KeysState::new(),
        }
    }

    pub fn recv_command(&mut self, tick: Tick, key_command_opt: Option<KeyCommand>) {
        self.keys_state.recv_command(tick, key_command_opt);
    }

    pub fn take_commands(&mut self, tick: Tick) -> Vec<KeyEvent> {
        self.keys_state.take_commands(tick)
    }
}

// KeysState
struct KeysState {
    w: KeyState,
    s: KeyState,
    a: KeyState,
    d: KeyState,
}

impl KeysState {
    fn new() -> Self {
        Self {
            w: KeyState::new(Key::W),
            s: KeyState::new(Key::S),
            a: KeyState::new(Key::A),
            d: KeyState::new(Key::D),
        }
    }

    fn recv_command(&mut self, tick: Tick, key_command_opt: Option<KeyCommand>) {
        if let Some(key_command) = key_command_opt {
            if let Some(w) = key_command.w {
                self.w.recv_stream(tick, w);
            } else {
                self.w.recv_none(tick);
            }
            if let Some(s) = key_command.s {
                self.s.recv_stream(tick, s);
            } else {
                self.s.recv_none(tick);
            }
            if let Some(a) = key_command.a {
                self.a.recv_stream(tick, a);
            } else {
                self.a.recv_none(tick);
            }
            if let Some(d) = key_command.d {
                self.d.recv_stream(tick, d);
            } else {
                self.d.recv_none(tick);
            }
        } else {
            self.w.recv_none(tick);
            self.s.recv_none(tick);
            self.a.recv_none(tick);
            self.d.recv_none(tick);
        }
    }

    fn take_commands(&mut self, tick: Tick) -> Vec<KeyEvent> {

        let mut output = Vec::new();

        self.w.take_commands(tick, &mut output);
        self.s.take_commands(tick, &mut output);
        self.a.take_commands(tick, &mut output);
        self.d.take_commands(tick, &mut output);

        output
    }
}

// KeyState

struct KeyState {
    key: Key,
    current_tick_opt: Option<Tick>,
    // VecDeque<(pressed, duration (ms), is_finished)>))
    durations: VecDeque<(bool, u16, bool)>,
}

impl KeyState {
    fn new(key: Key) -> Self {
        Self {
            key,
            current_tick_opt: None,
            durations: VecDeque::new()
        }
    }

    fn recv_stream(&mut self, tick: Tick, key_stream: KeyStream) {
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
                *back_duration += TICK_DURATION_MS;
                return;
            }
        }

        self.durations.push_back((false, TICK_DURATION_MS, false));
    }

    pub(crate) fn take_commands(&mut self, tick: Tick, output: &mut Vec<KeyEvent>) {
        if self.current_tick_opt != Some(tick) {
            panic!("take_commands called with incorrect tick!");
        }

        let mut last_pressed = false;

        while self.durations.len() > 1 { // leave the last one for the next tick
            let (pressed, duration, finished) = self.durations.pop_front().unwrap();
            if !finished {
                panic!("obviously not finished!");
            }

            if pressed {
                output.push(KeyEvent::Pressed(self.key, duration));
                last_pressed = true;
            } else {
                output.push(KeyEvent::Released(self.key));
                last_pressed = false;
            }
        }

        if let Some((pressed, duration, finished)) = self.durations.front() {
            if *finished {
                panic!("last one should not be finished!");
            }

            if *pressed {
                output.push(KeyEvent::Held(self.key, *duration));
            } else {
                if last_pressed {
                    output.push(KeyEvent::Released(self.key));
                }
            }
        }
    }
}

// KeyEvent
pub enum KeyEvent {
    Pressed(Key, u16),
    Held(Key, u16),
    Released(Key),
}

mod tests {
    use super::*;

    #[test]
    fn test_1() {

        let key_stream = KeyStream::new(true);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);

        assert_eq!(key_state.durations.len(), 1);
        assert_eq!(key_state.durations[0], (true, 40, false));
    }

    #[test]
    fn test_2() {

        let key_stream = KeyStream::new(false);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);

        assert_eq!(key_state.durations.len(), 1);
        assert_eq!(key_state.durations[0], (false, 40, false));
    }

    #[test]
    fn test_3() {

        let mut key_stream = KeyStream::new(true);
        key_stream.add_duration(28);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);

        assert_eq!(key_state.durations.len(), 2);
        assert_eq!(key_state.durations, [(true, 28, true), (false, TICK_DURATION_MS - 28, false)]);
    }

    #[test]
    fn test_4() {

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_none(0);

        assert_eq!(key_state.durations.len(), 1);
        assert_eq!(key_state.durations, [(false, TICK_DURATION_MS, false)]);
    }

    #[test]
    fn test_5() {

        let mut key_stream = KeyStream::new(true);
        key_stream.add_duration(28);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);
        key_state.recv_none(1);

        assert_eq!(key_state.durations.len(), 2);
        assert_eq!(key_state.durations, [(true, 28, true), (false, (TICK_DURATION_MS + TICK_DURATION_MS - 28), false)]);
    }

    #[test]
    fn test_6() {

        let mut key_stream = KeyStream::new(true);
        key_stream.add_duration(28);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_none(0);
        key_state.recv_stream(1, key_stream);


        assert_eq!(key_state.durations.len(), 3);
        assert_eq!(key_state.durations, [(false, TICK_DURATION_MS, true), (true, 28, true), (false, TICK_DURATION_MS - 28, false)]);
    }

    #[test]
    fn test_7() {

        let mut key_stream = KeyStream::new(false);
        key_stream.add_duration(28);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_none(0);
        key_state.recv_stream(1, key_stream);


        assert_eq!(key_state.durations.len(), 2);
        assert_eq!(key_state.durations, [(false, TICK_DURATION_MS + 28, true), (true, TICK_DURATION_MS - 28, false)]);
    }

    #[test]
    fn test_8() {

        let mut key_stream = KeyStream::new(false);
        key_stream.add_duration(28);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);
        key_state.recv_none(1);

        assert_eq!(key_state.durations.len(), 3);
        assert_eq!(key_state.durations, [(false, 28, true), (true, TICK_DURATION_MS - 28, true), (false, TICK_DURATION_MS, false)]);
    }

    #[test]
    fn test_9() {

        let mut key_stream = KeyStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);
        key_state.recv_none(1);

        assert_eq!(key_state.durations.len(), 4);
        assert_eq!(key_state.durations, [(true, 21, true), (false, 13, true), (true, 5, true), (false, (TICK_DURATION_MS + TICK_DURATION_MS - 21 - 13 - 5), false)]);
    }

    #[test]
    fn test_10() {

        let mut key_stream = KeyStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_none(0);
        key_state.recv_stream(1, key_stream);

        assert_eq!(key_state.durations.len(), 5);
        assert_eq!(key_state.durations, [(false, TICK_DURATION_MS, true), (true, 21, true), (false, 13, true), (true, 5, true), (false, TICK_DURATION_MS - 21 - 13 - 5, false)]);
    }

    #[test]
    fn test_11() {

        let mut key_stream = KeyStream::new(false);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_none(0);
        key_state.recv_stream(1, key_stream);

        assert_eq!(key_state.durations.len(), 4);
        assert_eq!(key_state.durations, [(false, TICK_DURATION_MS + 21, true), (true, 13, true), (false, 5, true), (true, TICK_DURATION_MS - 21 - 13 - 5, false)]);
    }

    #[test]
    fn test_12() {

        let mut key_stream = KeyStream::new(false);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut key_state = KeyState::new(Key::A);
        key_state.recv_stream(0, key_stream);
        key_state.recv_none(1);

        assert_eq!(key_state.durations.len(), 5);
        assert_eq!(key_state.durations, [(false, 21, true), (true, 13, true), (false, 5, true), (true, TICK_DURATION_MS - 21 - 13 - 5, true), (false, TICK_DURATION_MS, false)]);
    }
}