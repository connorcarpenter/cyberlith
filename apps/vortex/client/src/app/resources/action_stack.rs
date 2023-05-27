use bevy_ecs::{
    prelude::{Commands, Entity, Query, Resource, World},
    system::SystemState,
};
use bevy_log::info;
use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};
use vortex_proto::components::FileSystemEntry;

use crate::app::components::file_system::FileSystemUiState;

pub enum Action {
    // A list of File Row entities to select
    SelectFiles(Vec<Entity>),
    // The directory entity to add the new File to, and the name of the new File
    NewFile(Entity, String),
    // The File Row entity to delete
    DeleteFile(Entity),
    // The File Row entity to rename, and the new name
    RenameFile(Entity, String),
}

#[derive(Resource)]
pub struct ActionStack {
    buffered_actions: Vec<Action>,
    undo_actions: Vec<Action>,
    redo_actions: Vec<Action>,
    undo_enabled: bool,
    redo_enabled: bool,
    buffered_check: bool,
}

impl ActionStack {
    pub fn new() -> Self {
        Self {
            buffered_actions: Vec::new(),
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            undo_enabled: true,
            redo_enabled: true,
            buffered_check: false,
        }
    }

    pub fn buffer_action(&mut self, action: Action) {
        self.buffered_actions.push(action);
    }

    pub fn has_undo(&self) -> bool {
        !self.undo_actions.is_empty() && self.undo_enabled
    }

    pub fn has_redo(&self) -> bool {
        !self.redo_actions.is_empty() && self.redo_enabled
    }

    pub fn undo_action(&mut self, world: &mut World) {
        if !self.undo_enabled {
            panic!("Undo is disabled!");
        }
        let Some(action) = self.undo_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        let reversed_action = self.execute_action(world, &action);

        self.redo_actions.push(reversed_action);

        self.check_top(world);
    }

    pub fn redo_action(&mut self, world: &mut World) {
        if !self.redo_enabled {
            panic!("Redo is disabled!");
        }
        let Some(action) = self.redo_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        let reversed_action = self.execute_action(world, &action);

        self.undo_actions.push(reversed_action);

        self.check_top(world);
    }

    pub fn execute_actions(&mut self, world: &mut World) {
        if self.buffered_check {
            self.check_top(world);
            self.buffered_check = false;
        }
        if self.buffered_actions.is_empty() {
            return;
        }
        let drained_actions: Vec<Action> = self.buffered_actions.drain(..).collect();
        for action in drained_actions {
            let reversed_action = self.execute_action(world, &action);
            self.undo_actions.push(reversed_action);
        }
        self.redo_actions.clear();

        self.check_top(world);
    }

    fn execute_action(&mut self, world: &mut World, action: &Action) -> Action {
        match &action {
            Action::SelectFiles(file_entities) => {
                let mut old_selected_files = Vec::new();
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Query<(Entity, &mut FileSystemUiState)>,
                )> = SystemState::new(world);
                let (mut commands, mut client, mut fs_query) = system_state.get_mut(world);

                // TODO: when shift/control is pressed, select multiple items

                // Deselect all selected files
                for (item_entity, mut ui_state) in fs_query.iter_mut() {
                    if ui_state.selected {
                        ui_state.selected = false;

                        old_selected_files.push(item_entity);

                        // Release Entity Authority
                        commands.entity(item_entity).release_authority(&mut client);
                    }
                }

                // Select all new selected files
                for file_entity in file_entities {
                    let Ok((_, mut ui_state)) = fs_query.get_mut(*file_entity) else {
                        panic!("Failed to get FileSystemUiState for row entity {:?}!", file_entity);
                    };

                    ui_state.selected = true;

                    // Request Entity Authority
                    commands.entity(*file_entity).request_authority(&mut client);
                }

                return Action::SelectFiles(old_selected_files);
            }
            Action::NewFile(dir_entity, new_file_name) => {
                todo!();
            }
            Action::DeleteFile(file_entity) => {
                todo!();
            }
            Action::RenameFile(file_entity, new_name) => {
                let mut system_state: SystemState<Query<&mut FileSystemEntry>> =
                    SystemState::new(world);
                let mut fs_query = system_state.get_mut(world);
                let Ok(mut fs_entry) = fs_query.get_mut(*file_entity) else {
                    panic!("Failed to get FileSystemEntry for row entity {:?}!", file_entity);
                };
                let old_name: String = fs_entry.name.to_string();
                *fs_entry.name = new_name.clone();
                return Action::RenameFile(*file_entity, old_name);
            }
        }
    }

    pub fn entity_update_auth_status(&mut self, entity: &Entity) {
        // if either the undo or redo stack's top entity is this entity, then we need to enable/disable undo based on new auth status

        if let Some(Action::SelectFiles(file_entities)) = self.undo_actions.last() {
            if file_entities.contains(entity) {
                self.buffered_check = true;
            }
        }

        if let Some(Action::SelectFiles(file_entities)) = self.redo_actions.last() {
            if file_entities.contains(entity) {
                self.buffered_check = true;
            }
        }
    }

    fn check_top(&mut self, world: &mut World) {
        self.check_top_undo(world);
        self.check_top_redo(world);
    }

    fn check_top_undo(&mut self, world: &mut World) {
        if let Some(Action::SelectFiles(file_entities)) = self.undo_actions.last() {
            self.undo_enabled = self.should_be_enabled(world, file_entities);
        } else {
            self.undo_enabled = true;
        }
    }

    fn check_top_redo(&mut self, world: &mut World) {
        if let Some(Action::SelectFiles(file_entities)) = self.redo_actions.last() {
            self.redo_enabled = self.should_be_enabled(world, file_entities);
        } else {
            self.redo_enabled = true;
        }
    }

    fn should_be_enabled(&self, world: &mut World, file_entities: &Vec<Entity>) -> bool {
        info!("Checking if undo/redo should be enabled");
        let mut system_state: SystemState<(
            Commands,
            Client,
        )> = SystemState::new(world);
        let (mut commands, client) = system_state.get_mut(world);

        for file_entity in file_entities {
            if let Some(EntityAuthStatus::Available) = commands.entity(*file_entity).authority(&client) {
                // enabled should continue being true
            } else {
                return false;
            }
        }
        return true;
    }
}
