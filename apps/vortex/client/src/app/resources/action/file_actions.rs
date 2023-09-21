
use bevy_ecs::{entity::Entity, prelude::Resource, world::World};

use crate::app::resources::action::{ActionStack, FileAction};
use crate::app::resources::file_manager::FileManager;

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
    pub fn execute_action(&mut self, world: &mut World, action: FileAction) {
        let file_manager = world.get_resource::<FileManager>().unwrap();
        let project_entity = file_manager.project_root_entity;
        self.action_stack.execute_action(world, Some(&project_entity), action);
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

    pub(crate) fn check_top(&mut self, world: &mut World) {
        self.action_stack.check_top(world);
    }
}
