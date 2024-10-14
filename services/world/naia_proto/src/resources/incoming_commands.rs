use std::collections::{HashMap, VecDeque};

use naia_bevy_shared::Tick;

use logging::info;

use crate::messages::{PlayerCommands, PlayerCommandStream, PlayerCommand};

const TICK_DURATION_MS: u16 = 40; // TODO: move to config

// Intended to be encapsulated within a Client or Server specific Resource!
pub struct IncomingCommands {
    map: HashMap<PlayerCommand, IncomingCommandStream>,
}

impl IncomingCommands {
    pub fn new() -> Self {

        let mut map = HashMap::new();
        map.insert(PlayerCommand::Up, IncomingCommandStream::new(PlayerCommand::Up));
        map.insert(PlayerCommand::Down, IncomingCommandStream::new(PlayerCommand::Down));
        map.insert(PlayerCommand::Left, IncomingCommandStream::new(PlayerCommand::Left));
        map.insert(PlayerCommand::Right, IncomingCommandStream::new(PlayerCommand::Right));

        Self {
            map
        }
    }

    pub fn recv_incoming_command(&mut self, tick: Tick, player_commands_opt: Option<PlayerCommands>) {

        info!("PlayerCommands -> IncomingCommands for Tick({:?})", tick);

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

    pub fn pop_incoming_command_events(&mut self, tick: Tick) -> Vec<PlayerCommandEvent> {

        info!("IncomingCommands -> PlayerCommandEvents for Tick({:?})", tick);

        let mut output = Vec::new();

        for (_command, stream) in self.map.iter_mut() {
            let commands = stream.pop_commands(tick);
            output.extend(commands);
        }

        output
    }
}

// IncomingCommandStream

struct IncomingCommandStream {
    command: PlayerCommand,

    // front is the oldest
    // back is the newest
    // (pressed, duration_milliseconds)
    durations: VecDeque<(Tick, Vec<(bool, u8)>)>,
}

impl IncomingCommandStream {
    fn new(command: PlayerCommand) -> Self {
        Self {
            command,
            durations: VecDeque::new()
        }
    }

    fn recv_stream(&mut self, tick: Tick, key_stream: &PlayerCommandStream) {

        let mut pressed = key_stream.start_pressed();
        let incoming_durations = key_stream.durations();
        let mut remaining_duration = TICK_DURATION_MS as u8;
        let mut outgoing_durations = Vec::new();

        for duration in incoming_durations {
            let duration = duration.get() as u8;
            if duration > remaining_duration {
                panic!("duration > remaining_duration!");
            } else {
                outgoing_durations.push((pressed, duration));
                pressed = !pressed;
                remaining_duration -= duration;
            }
        }

        if remaining_duration > 0 {
            outgoing_durations.push((pressed, remaining_duration));
        }

        self.durations.push_back((tick, outgoing_durations));
    }

    fn recv_none(&mut self, tick: Tick) {
        let outgoing_durations = vec![(false, TICK_DURATION_MS as u8)];
        self.durations.push_back((tick, outgoing_durations));
    }

    pub(crate) fn pop_commands(&mut self, tick: Tick) -> Vec<PlayerCommandEvent> {

        let mut output = Vec::new();

        let Some((front_tick, front_durations)) = self.durations.pop_front() else {
            panic!("IncomingCommandStream::pop_commands called with a Tick({:?}) that doesn't match any Tick in the history.", tick);
        };
        if front_tick != tick {
            panic!("IncomingCommandStream::pop_commands called with a Tick({:?}) that doesn't match the FrontTick({:?}.", tick, front_tick);
        }

        for (pressed, duration) in front_durations {
            if pressed {
                output.push(PlayerCommandEvent::Pressed(self.command, duration));
            } else {
                output.push(PlayerCommandEvent::Released(self.command, duration));
            }
        }

        output
    }
}

// KeyEvent
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum PlayerCommandEvent {
    // key, duration (ms)
    Pressed(PlayerCommand, u8),
    // key, duration (ms)
    Released(PlayerCommand, u8),
}

mod tests {
    use super::*;

    #[test]
    fn test_1() {

        let key_stream = PlayerCommandStream::new(true);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_2() {

        let key_stream = PlayerCommandStream::new(false);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        
        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_3() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        
        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 2);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 28));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8 - 28));
    }

    #[test]
    fn test_4() {

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_none(0);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_5() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 2);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 28));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8 - 28));

        let command_events = command_stream.pop_commands(1);
        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

    }

    #[test]
    fn test_6() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 2);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 28));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8 - 28));
    }

    #[test]
    fn test_7() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 2);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, 28));
        assert_eq!(command_events[1], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8 - 28));
    }

    #[test]
    fn test_8() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(28);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 2);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, 28));
        assert_eq!(command_events[1], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8 - 28));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_9() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 4);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, 13));
        assert_eq!(command_events[2], PlayerCommandEvent::Pressed(PlayerCommand::Up, 5));
        assert_eq!(command_events[3], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8 - 21 - 13 - 5));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_10() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 4);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, 13));
        assert_eq!(command_events[2], PlayerCommandEvent::Pressed(PlayerCommand::Up, 5));
        assert_eq!(command_events[3], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8 - 21 - 13 - 5));
    }

    #[test]
    fn test_11() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_none(0);
        command_stream.recv_stream(1, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 4);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Pressed(PlayerCommand::Up, 13));
        assert_eq!(command_events[2], PlayerCommandEvent::Released(PlayerCommand::Up, 5));
        assert_eq!(command_events[3], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8 - 21 - 13 - 5));
    }

    #[test]
    fn test_12() {

        let mut key_stream = PlayerCommandStream::new(false);
        key_stream.add_duration(21);
        key_stream.add_duration(13);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_none(1);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 4);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Pressed(PlayerCommand::Up, 13));
        assert_eq!(command_events[2], PlayerCommandEvent::Released(PlayerCommand::Up, 5));
        assert_eq!(command_events[3], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8 - 21 - 13 - 5));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_13() {

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_none(0);
        command_stream.recv_none(1);
        command_stream.recv_none(2);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(2);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_14() {

        let key_stream = PlayerCommandStream::new(false);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_stream(1, &key_stream);
        command_stream.recv_stream(2, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(2);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Released(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_15() {

        let key_stream = PlayerCommandStream::new(true);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_stream(1, &key_stream);
        command_stream.recv_stream(2, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8));

        let command_events = command_stream.pop_commands(2);

        assert_eq!(command_events.len(), 1);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8));
    }

    #[test]
    fn test_16() {

        let mut key_stream = PlayerCommandStream::new(true);
        key_stream.add_duration(21);
        key_stream.add_duration(5);

        let mut command_stream = IncomingCommandStream::new(PlayerCommand::Up);
        command_stream.recv_stream(0, &key_stream);
        command_stream.recv_stream(1, &key_stream);
        command_stream.recv_stream(2, &key_stream);

        let command_events = command_stream.pop_commands(0);

        assert_eq!(command_events.len(), 3);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, 5));
        assert_eq!(command_events[2], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8 - 21 - 5));

        let command_events = command_stream.pop_commands(1);

        assert_eq!(command_events.len(), 3);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, 5));
        assert_eq!(command_events[2], PlayerCommandEvent::Pressed(PlayerCommand::Up, TICK_DURATION_MS as u8 - 21 - 5));

        let command_events = command_stream.pop_commands(2);

        assert_eq!(command_events.len(), 3);
        assert_eq!(command_events[0], PlayerCommandEvent::Pressed(PlayerCommand::Up, 21));
        assert_eq!(command_events[1], PlayerCommandEvent::Released(PlayerCommand::Up, 5));
    }
}