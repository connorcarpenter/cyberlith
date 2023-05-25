use bevy_ecs::prelude::{Entity, Resource};

pub enum Action {
    RenameFile,
}

#[derive(Resource)]
pub struct ActionStack {
    executed_actions: Vec<Action>,
    undone_actions: Vec<Action>,
}

impl ActionStack {
    pub fn new() -> Self {
        Self {
            executed_actions: Vec::new(),
            undone_actions: Vec::new(),
        }
    }

    pub fn execute_action(&mut self, action: Action) {
        match &action {
            Action::RenameFile => {
                todo!("Execute rename file");
            }
        }

        self.executed_actions.push(action);
    }

    pub fn has_undo(&self) -> bool {
        !self.executed_actions.is_empty()
    }

    pub fn has_redo(&self) -> bool {
        !self.undone_actions.is_empty()
    }

    pub fn undo_action(&mut self) {
        let Some(action) = self.executed_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        match &action {
            Action::RenameFile => {
                todo!("Undo rename file");
            }
        }

        self.undone_actions.push(action);
    }

    pub fn redo_action(&mut self) {
        let Some(action) = self.undone_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        match &action {
            Action::RenameFile => {
                todo!("Redo rename file");
            }
        }

        self.executed_actions.push(action);
    }
}