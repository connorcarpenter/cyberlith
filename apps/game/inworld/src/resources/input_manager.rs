use std::default::Default;

use bevy_ecs::{system::{Res, ResMut}, prelude::Resource};

use game_engine::{logging::warn, input::{Input, Key}, naia::{GameInstant, CommandHistory, Tick}, world::{messages::{KeyCommand, KeyStream}, WorldClient}};

use crate::resources::Global;

#[derive(Resource)]
pub struct InputManager {
    current_tick_opt: Option<Tick>,
    command_state_opt: Option<CommandState>,

    command_history: CommandHistory<KeyCommand>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            current_tick_opt: None,
            command_state_opt: None,
            command_history: CommandHistory::default(),
        }
    }
}

impl InputManager {

    // used as a system
    pub fn key_input(
        mut me: ResMut<Self>,
        client: WorldClient,
        global: Res<Global>,
        input: Res<Input>,
    ) {
        if global.owned_entity.is_none() {
            return;
        }

        let Some(client_instant) = client.client_instant() else {
            return;
        };

        if me.command_state_opt.is_none() {
            me.command_state_opt = Some(CommandState::new(client_instant));
        }
        let command_state = me.command_state_opt.as_mut().unwrap();
        command_state.recv_input(client_instant, &input);
    }

    pub fn take_command(&mut self, client_instant: GameInstant) -> Option<KeyCommand> {
        self.command_state_opt.as_mut()?.take_command(client_instant)
    }

    pub fn take_command_replays(&mut self, server_tick: Tick) -> Vec<(Tick, KeyCommand)> {

        // TODO: fix this?
        let modified_server_tick = server_tick.wrapping_sub(1);

        self.command_history.replays(&modified_server_tick)
    }

    pub fn save_to_command_history(&mut self, client_tick: Tick, command: &KeyCommand) {
        {
            if !self.command_history.can_insert(&client_tick) {
                // History is full, should this be possible??
                panic!(
                    "Command History is full, cannot insert command for tick: {:?}",
                    client_tick
                );
            }

            // Record command
            self.command_history.insert(client_tick, command.clone());
        }
    }
}

// Command State
struct CommandState {
    w: StreamState,
    s: StreamState,
    a: StreamState,
    d: StreamState,
}

impl CommandState {
    fn new(now: GameInstant) -> Self {
        Self {
            w: StreamState::new(now),
            s: StreamState::new(now),
            a: StreamState::new(now),
            d: StreamState::new(now),
        }
    }

    fn is_empty(&self) -> bool {
        self.w.is_empty() && self.s.is_empty() && self.a.is_empty() && self.d.is_empty()
    }

    fn take_command(&mut self, client_instant: GameInstant) -> Option<KeyCommand> {

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

    fn recv_input(&mut self, client_instant: GameInstant, input: &Input) {
        self.w.recv_input(client_instant, input.is_pressed(Key::W));
        self.s.recv_input(client_instant, input.is_pressed(Key::S));
        self.a.recv_input(client_instant, input.is_pressed(Key::A));
        self.d.recv_input(client_instant, input.is_pressed(Key::D));
    }
}

// Stream State

struct StreamState {
    start_pressed: bool,
    pressed: bool,
    durations: Vec<u8>,
    last_toggle: GameInstant,
}

impl StreamState {

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
