use bevy_ecs::prelude::{Res, ResMut};

use game_engine::{
    input::{Input, Key},
    world::messages::KeyCommand,
};

use crate::resources::Global;

pub fn key_input(
    mut global: ResMut<Global>,
    input: Res<Input>
) {
    if global.owned_entity.is_none() {
        return;
    }

    let w = input.is_pressed(Key::W);
    let s = input.is_pressed(Key::S);
    let a = input.is_pressed(Key::A);
    let d = input.is_pressed(Key::D);

    if let Some(command) = &mut global.queued_command {
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
        global.queued_command = Some(key_command);
    }
}
