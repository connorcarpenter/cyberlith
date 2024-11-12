use naia_bevy_shared::Message;

use crate::types::Direction;

#[derive(Message)]
pub struct PlayerCommands {
    look_opt: Option<Direction>,
    move_opt: Option<Direction>,
}

impl PlayerCommands {
    pub fn new() -> Self {
        Self {
            look_opt: None,
            move_opt: None,
        }
    }

    pub fn get_look(&self) -> Option<Direction> {
        self.look_opt
    }

    pub fn set_look(&mut self, direction: Direction) {
        self.look_opt = Some(direction);
    }

    pub fn get_move(&self) -> Option<Direction> {
        self.move_opt
    }

    pub fn set_move(&mut self, direction: Direction) {
        self.move_opt = Some(direction);
    }

    pub fn merge_newer(&mut self, newer: &PlayerCommands) {
        if let Some(newer_look) = newer.get_look() {
            self.set_look(newer_look);
        }
        if let Some(newer_move) = newer.get_move() {
            self.set_move(newer_move);
        }
    }
}
