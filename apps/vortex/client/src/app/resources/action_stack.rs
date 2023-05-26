use bevy_ecs::{prelude::{Entity, Query, Resource, World}, system::SystemState};
use vortex_proto::components::FileSystemEntry;

pub enum Action {
    RenameFile(Entity, String),
}

#[derive(Resource)]
pub struct ActionStack {
    buffered_actions: Vec<Action>,
    executed_actions: Vec<Action>,
    undone_actions: Vec<Action>,
}

impl ActionStack {
    pub fn new() -> Self {
        Self {
            buffered_actions: Vec::new(),
            executed_actions: Vec::new(),
            undone_actions: Vec::new(),
        }
    }

    pub fn execute_actions(&mut self, world: &mut World) {
        let drained_actions: Vec<Action> = self.buffered_actions.drain(..).collect();
        for action in drained_actions {
            let reversed_action = self.execute_action(world, &action);
            self.executed_actions.push(reversed_action);
        }
    }

    fn execute_action(&mut self, world: &mut World, action: &Action) -> Action {
        match &action {
            Action::RenameFile(row_entity, new_name) => {
                let mut system_state: SystemState<Query<&mut FileSystemEntry>> =
                    SystemState::new(world);
                let mut fs_query = system_state.get_mut(world);
                let Ok(mut fs_entry) = fs_query.get_mut(*row_entity) else {
                    panic!("Failed to get FileSystemEntry for row entity {:?}!", row_entity);
                };
                let old_name: String = fs_entry.name.to_string();
                *fs_entry.name = new_name.clone();
                return Action::RenameFile(*row_entity, old_name);
            }
        }
    }

    pub fn buffer_action(&mut self, action: Action) {
        self.buffered_actions.push(action);
    }

    pub fn has_undo(&self) -> bool {
        !self.executed_actions.is_empty()
    }

    pub fn has_redo(&self) -> bool {
        !self.undone_actions.is_empty()
    }

    pub fn undo_action(&mut self, world: &mut World) {
        let Some(action) = self.executed_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        let reversed_action = self.execute_action(world, &action);

        self.undone_actions.push(reversed_action);
    }

    pub fn redo_action(&mut self, world: &mut World) {
        let Some(action) = self.undone_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        let reversed_action = self.execute_action(world, &action);

        self.executed_actions.push(reversed_action);
    }
}