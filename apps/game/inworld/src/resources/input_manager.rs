use std::default::Default;

use bevy_ecs::{system::{Res, ResMut}, prelude::Resource};

use game_engine::{input::{Input, Key}, naia::CommandHistory, world::messages::KeyCommand};

use crate::resources::Global;

#[derive(Resource)]
pub struct InputManager {
    pub queued_command: Option<KeyCommand>,
    pub command_history: CommandHistory<KeyCommand>,
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
}