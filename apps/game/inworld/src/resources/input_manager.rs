use std::default::Default;

use bevy_ecs::{system::{Res, ResMut}, prelude::Resource};

use game_engine::{input::{Input, Key}, naia::{CommandHistory, Tick}, world::messages::KeyCommand};

use crate::resources::Global;

#[derive(Resource)]
pub struct InputManager {
    queued_command: Option<KeyCommand>,
    command_history: CommandHistory<KeyCommand>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            queued_command: None,
            command_history: CommandHistory::default(),
        }
    }
}

impl InputManager {

    // used as a system
    pub fn key_input(
        global: Res<Global>,
        mut input_manager: ResMut<InputManager>,
        input: Res<Input>,
    ) {
        if global.owned_entity.is_none() {
            return;
        }

        let w = input.is_pressed(Key::W);
        let s = input.is_pressed(Key::S);
        let a = input.is_pressed(Key::A);
        let d = input.is_pressed(Key::D);

        if let Some(command) = &mut input_manager.queued_command {
            if w {
                command.w = true;
            }
            if s {
                command.s = true;
            }
            if a {
                command.a = true;
            }
            if d {
                command.d = true;
            }
        } else {
            let key_command = KeyCommand::new(w, s, a, d);
            input_manager.queued_command = Some(key_command);
        }
    }

    pub fn take_queued_command(&mut self) -> Option<KeyCommand> {
        self.queued_command.take()
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