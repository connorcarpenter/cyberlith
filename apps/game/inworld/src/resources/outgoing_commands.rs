use game_engine::{world::messages::{KeyCommand, KeyStream}, naia::GameInstant, logging::warn, input::{Input, Key}};

// Outgoing Commands
pub struct OutgoingCommands {
    w: OutgoingCommandStream,
    s: OutgoingCommandStream,
    a: OutgoingCommandStream,
    d: OutgoingCommandStream,
}

impl OutgoingCommands {
    pub fn new(now: GameInstant) -> Self {
        Self {
            w: OutgoingCommandStream::new(now),
            s: OutgoingCommandStream::new(now),
            a: OutgoingCommandStream::new(now),
            d: OutgoingCommandStream::new(now),
        }
    }

    fn is_empty(&self) -> bool {
        self.w.is_empty() && self.s.is_empty() && self.a.is_empty() && self.d.is_empty()
    }

    pub fn recv_key_input(&mut self, client_instant: GameInstant, input: &Input) {
        self.w.recv_input(client_instant, input.is_pressed(Key::W));
        self.s.recv_input(client_instant, input.is_pressed(Key::S));
        self.a.recv_input(client_instant, input.is_pressed(Key::A));
        self.d.recv_input(client_instant, input.is_pressed(Key::D));
    }

    pub fn pop_outgoing_command(&mut self, client_instant: GameInstant) -> Option<KeyCommand> {

        if self.is_empty() {
            self.flush_all(client_instant);
            return None;
        }

        let mut output = KeyCommand::new();
        if !self.w.is_empty() {
            output.w = Some(self.w.to_key_stream());
        }
        if !self.s.is_empty() {
            output.s = Some(self.s.to_key_stream());
        }
        if !self.a.is_empty() {
            output.a = Some(self.a.to_key_stream());
        }
        if !self.d.is_empty() {
            output.d = Some(self.d.to_key_stream());
        }

        self.flush_all(client_instant);

        Some(output)
    }

    fn flush_all(&mut self, client_instant: GameInstant) {
        self.w.flush(client_instant);
        self.s.flush(client_instant);
        self.a.flush(client_instant);
        self.d.flush(client_instant);
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

    fn to_key_stream(&mut self) -> KeyStream {
        let mut output = KeyStream::new(self.start_pressed);
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