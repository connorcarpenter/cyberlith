use bevy_ecs::{entity::Entity, prelude::Resource, world::World};

use crate::app::resources::{
    action::{ActionStack, FileAction},
    file_manager::FileManager,
};

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
    pub fn execute_file_action(&mut self, world: &mut World, action: FileAction) {
        let file_manager = world.get_resource::<FileManager>().unwrap();
        let project_entity = file_manager.project_root_entity;
        let reversed_actions = self
            .action_stack
            .execute_action(world, project_entity, action);
        self.action_stack
            .post_action_execution(world, reversed_actions);
    }

    pub(crate) fn has_undo(&self) -> bool {
        self.action_stack.has_undo()
    }

    pub(crate) fn has_redo(&self) -> bool {
        self.action_stack.has_redo()
    }

    pub(crate) fn undo_action(&mut self, world: &mut World, project_root_entity: Entity) {
        let action = self.action_stack.pop_undo();
        let reversed_actions = self
            .action_stack
            .execute_action(world, project_root_entity, action);
        self.action_stack.post_execute_undo(world, reversed_actions);
    }

    pub(crate) fn redo_action(&mut self, world: &mut World, project_root_entity: Entity) {
        let action = self.action_stack.pop_redo();
        let reversed_actions = self
            .action_stack
            .execute_action(world, project_root_entity, action);
        self.action_stack.post_execute_redo(world, reversed_actions);
    }

    pub(crate) fn entity_update_auth_status(&mut self, entity: &Entity) {
        self.action_stack.entity_update_auth_status(entity);
    }

    pub(crate) fn check_top(&mut self, world: &mut World) {
        self.action_stack.check_top(world);
    }
}
