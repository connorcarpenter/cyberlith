use bevy_ecs::{entity::Entity, prelude::Resource, world::World};

use crate::app::resources::action::{ActionStack, FileAction};

#[derive(Resource)]
pub struct FileActions {
    action_stack: ActionStack<FileAction>,
}

impl Default for FileActions {
    fn default() -> Self {
        Self {
            action_stack: ActionStack::default(),
        }
    }
}

impl FileActions {
    pub fn buffer_action(&mut self, action: FileAction) {
        self.action_stack.buffer_action(action);
    }

    pub(crate) fn has_undo(&self) -> bool {
        self.action_stack.has_undo()
    }

    pub(crate) fn has_redo(&self) -> bool {
        self.action_stack.has_redo()
    }

    pub(crate) fn undo_action(&mut self, world: &mut World, project_root_entity: Option<&Entity>) {
        self.action_stack.undo_action(world, project_root_entity);
    }

    pub(crate) fn redo_action(&mut self, world: &mut World, project_root_entity: Option<&Entity>) {
        self.action_stack.redo_action(world, project_root_entity);
    }

    pub(crate) fn entity_update_auth_status(&mut self, entity: &Entity) {
        self.action_stack.entity_update_auth_status(entity);
    }

    pub(crate) fn execute_actions(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        self.action_stack.execute_actions(world, entity_opt);
    }
}