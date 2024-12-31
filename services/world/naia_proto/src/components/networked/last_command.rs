use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

use crate::{types::Direction, messages::PlayerCommands};

#[derive(Component, Replicate)]
pub struct NetworkedLastCommand {
    last_move: Property<Option<Direction>>,
}

impl NetworkedLastCommand {
    pub fn new(dir_opt: Option<Direction>) -> Self {
        Self::new_complete(dir_opt)
    }

    pub fn get(&self) -> Option<Direction> {
        *self.last_move
    }

    fn set(&mut self, dir_opt: Option<Direction>) {
        if *self.last_move != dir_opt {
            *self.last_move = dir_opt;
        }
    }

    pub fn recv_command(&mut self, player_commands_opt: Option<PlayerCommands>) {
        if player_commands_opt.is_none() {
            self.set(None);
            return;
        }

        let player_commands = player_commands_opt.unwrap();
        let move_dir = player_commands.get_move();
        self.set(move_dir);
        return;
    }

    pub fn to_player_commands(&self) -> Option<PlayerCommands> {

        let last_move = self.get();
        if last_move.is_none() {
            return None;
        }

        let mut output = PlayerCommands::new();
        output.set_move(last_move.unwrap());
        Some(output)
    }
}
