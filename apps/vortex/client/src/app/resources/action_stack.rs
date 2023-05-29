
use bevy_ecs::{
    prelude::{Commands, Entity, Query, Resource, World},
    system::{SystemState, Res},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus, ReplicationConfig};
use vortex_proto::components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::app::{components::file_system::{FileSystemParent, FileSystemUiState}, resources::global::Global, slim_tree::SlimTree, systems::file_post_process};

pub enum Action {
    // A list of File Row entities to select
    SelectEntries(Vec<Entity>),
    // The directory entity to add the new Entry to, the name of the new Entry, it's Kind, and a list of child Entries to create
    NewEntry(Option<Entity>, String, EntryKind, Option<Vec<SlimTree>>),
    // The File Row entity to delete, and a list of entities to select after deleted
    DeleteEntry(Entity, Option<Vec<Entity>>),
    // The File Row entity to rename, and the new name
    RenameEntry(Entity, String),
}

impl Action {
    pub(crate) fn migrate_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        match self {
            Action::SelectEntries(entities) => {
                for entity in entities {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            Action::NewEntry(entity_opt, _, _, _) => {
                if let Some(entity) = entity_opt {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            Action::DeleteEntry(entity, entities_opt) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
                if let Some(entities) = entities_opt {
                    for entity in entities {
                        if *entity == old_entity {
                            *entity = new_entity;
                        }
                    }
                }
            }
            Action::RenameEntry(entity, _) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
            }
        }
    }
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
            Action::SelectEntries(file_entities) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Query<(Entity, &mut FileSystemUiState)>,
                )> = SystemState::new(world);
                let (mut commands, mut client, mut ui_query) = system_state.get_mut(world);

                // TODO: when shift/control is pressed, select multiple items

                // Deselect all selected files
                let old_selected_files =
                    Self::deselect_all_selected_files(&mut commands, &mut client, &mut ui_query);

                // Select all new selected files
                Self::select_files(&mut commands, &mut client, &mut ui_query, file_entities);

                return Action::SelectEntries(old_selected_files);
            }
            Action::NewEntry(parent_entity_opt, new_file_name, entry_kind, entry_contents_opt) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Res<Global>,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<&mut FileSystemParent>,
                )> = SystemState::new(world);
                let (mut commands, mut client, global, mut ui_query, mut parent_query) =
                    system_state.get_mut(world);

                let old_selected_files =
                    Self::deselect_all_selected_files(&mut commands, &mut client, &mut ui_query);

                let mut parent = {
                    if let Some(parent_entity) = parent_entity_opt {
                        parent_query.get_mut(*parent_entity).unwrap()
                    } else {
                        parent_query.get_mut(global.project_root_entity).unwrap()
                    }
                };

                let entity_id = self.create_fs_entry(&mut commands, &mut client, &mut parent, parent_entity_opt, new_file_name, entry_kind, entry_contents_opt, true);

                system_state.apply(world);

                return Action::DeleteEntry(entity_id, Some(old_selected_files));
            }
            Action::DeleteEntry(file_entity, files_to_select_opt) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Res<Global>,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<(
                        &FileSystemEntry,
                        Option<&FileSystemChild>,
                        Option<&FileSystemRootChild>,
                    )>,
                    Query<&mut FileSystemParent>,
                )> = SystemState::new(world);
                let (mut commands, mut client, global, mut ui_query, fs_query, mut parent_query) =
                    system_state.get_mut(world);
                let (entry, fs_child_opt, fs_root_child_opt) = fs_query.get(*file_entity).unwrap();

                // get name of file
                let entry_name = entry.name.to_string();
                let entry_kind = *entry.kind;

                // get parent entity
                let parent_entity_opt: Option<Entity> = if let Some(fs_child) = fs_child_opt {
                    // get parent entity
                    let parent_entity = fs_child.parent_id.get(&client).unwrap();
                    // remove entity from parent
                    parent_query
                        .get_mut(parent_entity)
                        .unwrap()
                        .remove_child(file_entity);

                    Some(parent_entity)
                } else if let Some(_) = fs_root_child_opt {
                    // remove entity from root
                    parent_query
                        .get_mut(global.project_root_entity)
                        .unwrap()
                        .remove_child(file_entity);

                    None
                } else {
                    panic!(
                        "FileSystemEntry {:?} has neither FileSystemChild nor FileSystemRootChild!",
                        file_entity
                    );
                };

                let entry_contents_opt = {
                    match entry_kind {
                        EntryKind::File => None,
                        EntryKind::Directory => {
                            let (entries, entities_to_delete) = Self::convert_contents_to_slim_tree(&client, file_entity, &fs_query, &mut parent_query);

                            // delete all children
                            for entity in entities_to_delete {
                                commands.entity(entity).despawn();
                            }

                            Some(entries)
                        },
                    }
                };

                // actually delete the entry
                commands.entity(*file_entity).despawn();

                if let Some(files_to_select) = files_to_select_opt {
                    Self::select_files(&mut commands, &mut client, &mut ui_query, files_to_select);
                }

                system_state.apply(world);

                return Action::NewEntry(
                    parent_entity_opt,
                    entry_name,
                    entry_kind,
                    entry_contents_opt.map(|entries| entries.into_iter().map(|(_, tree)| tree).collect()));
            }
            Action::RenameEntry(file_entity, new_name) => {
                let mut system_state: SystemState<Query<&mut FileSystemEntry>> =
                    SystemState::new(world);
                let mut entry_query = system_state.get_mut(world);
                let Ok(mut file_entry) = entry_query.get_mut(*file_entity) else {
                    panic!("Failed to get FileSystemEntry for row entity {:?}!", file_entity);
                };
                let old_name: String = file_entry.name.to_string();
                *file_entry.name = new_name.clone();
                return Action::RenameEntry(*file_entity, old_name);
            }
        }
    }

    fn select_files(
        commands: &mut Commands,
        client: &mut Client,
        ui_query: &mut Query<(Entity, &mut FileSystemUiState)>,
        file_entities: &Vec<Entity>,
    ) {
        for file_entity in file_entities {
            let Ok((_, mut ui_state)) = ui_query.get_mut(*file_entity) else {
                panic!("Failed to get FileSystemUiState for row entity {:?}!", file_entity);
            };

            ui_state.selected = true;

            // Request Entity Authority
            commands.entity(*file_entity).request_authority(client);
        }
    }

    fn deselect_all_selected_files(
        commands: &mut Commands,
        client: &mut Client,
        ui_query: &mut Query<(Entity, &mut FileSystemUiState)>,
    ) -> Vec<Entity> {
        let mut old_selected_files = Vec::new();
        for (item_entity, mut ui_state) in ui_query.iter_mut() {
            if ui_state.selected {
                ui_state.selected = false;

                old_selected_files.push(item_entity);

                // Release Entity Authority
                commands.entity(item_entity).release_authority(client);
            }
        }
        old_selected_files
    }

    fn create_fs_entry(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        parent: &mut FileSystemParent,
        parent_entity_opt: &Option<Entity>,
        new_file_name: &str,
        entry_kind: &EntryKind,
        entry_contents_opt: &Option<Vec<SlimTree>>,
        ui_should_select: bool,
    ) -> Entity {
        info!("creating new entry: `{}`", new_file_name);

        let entity_id = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .id();

        let entry = FileSystemEntry::new(new_file_name, *entry_kind);

        // add FileSystemChild or FileSystemRootChild component
        if let Some(parent_entity) = parent_entity_opt {
            let mut child_component = FileSystemChild::new();
            child_component.parent_id.set(client, &parent_entity);
            commands.entity(entity_id).insert(child_component);
        } else {
            commands.entity(entity_id).insert(FileSystemRootChild);
        }

        // add UiState component
        file_post_process::insert_ui_state_component(
            commands,
            entity_id,
            ui_should_select,
        );

        if *entry.kind == EntryKind::Directory {
            let mut entry_parent_component = FileSystemParent::new();

            if let Some(entry_contents) = entry_contents_opt {
                for sub_tree in entry_contents {

                    let new_entity = self.create_fs_entry(
                        commands,
                        client,
                        &mut entry_parent_component,
                        &Some(entity_id),
                        &sub_tree.name,
                        &sub_tree.kind,
                        &sub_tree.children,
                        false,
                    );
                    let old_entity = sub_tree.entity;
                    self.migrate_undo_entities(old_entity, new_entity);
                }
            }

            // add FileSystemParent component
            commands.entity(entity_id).insert(entry_parent_component);
        }

        // add child to parent
        file_post_process::parent_add_child_entry(parent, &entry, entity_id);

        // add FileSystemEntry component
        commands.entity(entity_id).insert(entry);

        entity_id
    }

    pub fn entity_update_auth_status(&mut self, entity: &Entity) {
        // if either the undo or redo stack's top entity is this entity, then we need to enable/disable undo based on new auth status

        if let Some(Action::SelectEntries(file_entities)) = self.undo_actions.last() {
            if file_entities.contains(entity) {
                self.buffered_check = true;
            }
        }

        if let Some(Action::SelectEntries(file_entities)) = self.redo_actions.last() {
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
        if let Some(Action::SelectEntries(file_entities)) = self.undo_actions.last() {
            self.undo_enabled = self.should_be_enabled(world, file_entities);
        } else {
            self.undo_enabled = true;
        }
    }

    fn check_top_redo(&mut self, world: &mut World) {
        if let Some(Action::SelectEntries(file_entities)) = self.redo_actions.last() {
            self.redo_enabled = self.should_be_enabled(world, file_entities);
        } else {
            self.redo_enabled = true;
        }
    }

    fn should_be_enabled(&self, world: &mut World, file_entities: &Vec<Entity>) -> bool {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, client) = system_state.get_mut(world);

        for file_entity in file_entities {
            if let Some(EntityAuthStatus::Available) =
                commands.entity(*file_entity).authority(&client)
            {
                // enabled should continue being true
            } else {
                return false;
            }
        }
        return true;
    }

    fn convert_contents_to_slim_tree(
        client: &Client,
        parent_entity: &Entity,
        fs_query: &Query<(&FileSystemEntry, Option<&FileSystemChild>, Option<&FileSystemRootChild>)>,
        parent_query: &mut Query<&mut FileSystemParent>
    ) -> (Vec<(Entity, SlimTree)>, Vec<Entity>) {

        let mut entities_to_delete = Vec::new();
        let mut trees = Vec::new();

        if let Ok(parent) = parent_query.get(*parent_entity) {
            let children_entities = parent.get_children();
            for child_entity in children_entities {
                let (child_entry, _, _) = fs_query.get(child_entity).unwrap();
                let slim_tree = SlimTree::new(child_entity, child_entry.name.to_string(), *child_entry.kind);
                trees.push((child_entity, slim_tree));
                entities_to_delete.push(child_entity);
            }

            for (entry_entity, tree) in trees.iter_mut() {
                let (subtree, sub_entities_to_delete) = Self::convert_contents_to_slim_tree(client, entry_entity, fs_query, parent_query);
                if subtree.len() > 0 {
                    tree.children = Some(subtree.into_iter().map(|(_, child_tree)| child_tree).collect());
                }
                entities_to_delete.extend(sub_entities_to_delete);
            }
        }

        (trees, entities_to_delete)
    }
    fn migrate_undo_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action in self.undo_actions.iter_mut() {
            action.migrate_entities(old_entity, new_entity);
        }
    }
}
