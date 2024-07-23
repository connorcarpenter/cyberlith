use bevy_ecs::prelude::{Res, ResMut};

use game_engine::{input::{Input, Key}, world::{WorldClient, messages::KeyCommand}};

use crate::resources::Global;

pub fn key_input(
    client: WorldClient,
    mut global: ResMut<Global>,
    input: Res<Input>,
) {
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
    } else if let Some(owned_entity) = &global.owned_entity {
        let mut key_command = KeyCommand::new(w, s, a, d);
        key_command.entity.set(&client, &owned_entity.confirmed);
        global.queued_command = Some(key_command);
    }
}