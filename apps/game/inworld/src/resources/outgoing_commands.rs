use std::collections::HashMap;

use game_engine::{world::messages::{PlayerCommand, PlayerCommands, PlayerCommandStream}, naia::GameInstant, logging::warn, input::{Input, Key}};

// Outgoing Commands
pub struct OutgoingCommands {
    map: HashMap<PlayerCommand, (Key, OutgoingCommandStream)>,
}

impl OutgoingCommands {
    pub fn new(now: GameInstant) -> Self {

        let mut map = HashMap::new();
        map.insert(PlayerCommand::Forward, (Key::W, OutgoingCommandStream::new(now)));
        map.insert(PlayerCommand::Backward, (Key::S, OutgoingCommandStream::new(now)));
        map.insert(PlayerCommand::Left, (Key::A, OutgoingCommandStream::new(now)));
        map.insert(PlayerCommand::Right, (Key::D, OutgoingCommandStream::new(now)));

        Self {
            map,
        }
    }

    fn is_empty(&self) -> bool {
        self.map.values().all(|(_, stream)| stream.is_empty())
    }

    pub fn recv_key_input(&mut self, client_instant: GameInstant, input: &Input) {
        for (_command, (key, stream)) in self.map.iter_mut() {
            stream.recv_input(client_instant, input.is_pressed(*key));
        }
    }

    pub fn pop_outgoing_command(&mut self, client_instant: GameInstant) -> Option<PlayerCommands> {

        if self.is_empty() {
            self.flush_all(client_instant);
            return None;
        }

        let mut output = PlayerCommands::new();
        for (command, (_key, stream)) in self.map.iter_mut() {
            if !stream.is_empty() {
                output.set(*command, stream.to_key_stream());
            }
        }

        self.flush_all(client_instant);

        Some(output)
    }

    fn flush_all(&mut self, client_instant: GameInstant) {
        for (_command, (_key, stream)) in self.map.iter_mut() {
            stream.flush(client_instant);
        }
    }
}

// Stream State

struct OutgoingCommandStream {
    start_pressed: bool,
    pressed: bool,
    durations: Vec<u8>,
    last_toggle: GameInstant,
}

impl OutgoingCommandStream {

    fn new(now: GameInstant) -> Self {
        Self {
            start_pressed: false,
            durations: Vec::new(),

            pressed: false,
            last_toggle: now,
        }
    }

    fn is_empty(&self) -> bool {
        !self.start_pressed && self.durations.is_empty()
    }

    fn recv_input(&mut self, client_instant: GameInstant, pressed: bool) {
        if self.pressed != pressed {
            self.pressed = pressed;

            let mut duration = self.last_toggle.offset_from(&client_instant);

            self.last_toggle = client_instant;

            if duration > 63 {
                warn!("Attempted to add duration > 63ms! ({}ms)", duration);
                duration = 63;
            }
            self.durations.push(duration as u8);


        }
    }

    fn to_key_stream(&mut self) -> PlayerCommandStream {
        let mut output = PlayerCommandStream::new(self.start_pressed);
        for duration in &self.durations {
            output.add_duration(*duration);
        }

        output
    }

    fn flush(&mut self, client_instant: GameInstant) {
        self.start_pressed = self.pressed;
        self.durations.clear();
        self.last_toggle = client_instant;
    }
}