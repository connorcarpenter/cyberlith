use std::collections::HashSet;

use bevy_ecs::{
    prelude::{Commands, Entity, Query, Resource, World},
    system::{Res, ResMut, SystemState},
};
use bevy_log::info;
use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus, ReplicationConfig};

use vortex_proto::components::{ChangelistEntry, EntryKind, FileSystemEntry, HasParent, NoParent};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
    resources::{file_tree::FileTree, global::Global, tab_manager::TabManager},
    systems::file_post_process,
};

pub enum Action {
    // A list of File Row entities to select
    SelectEntries(Vec<Entity>),
    // The directory entity to add the new Entry to, the name of the new Entry, it's Kind, an older Entity it was associated with if necessary, and a list of child Entries to create
    NewEntry(
        Option<Entity>,
        String,
        EntryKind,
        Option<Entity>,
        Option<Vec<FileTree>>,
    ),
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
            Action::NewEntry(entity_opt, _, _, entity_opt_2, _) => {
                if let Some(entity) = entity_opt {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
                if let Some(entity) = entity_opt_2 {
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
                    Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
                )> = SystemState::new(world);
                let (mut commands, mut client, mut fs_query, mut cl_query) =
                    system_state.get_mut(world);

                // TODO: when shift/control is pressed, select multiple items

                // Deselect all selected files, select the new selected files
                let (deselected_row_entities, mut file_entries_to_release) =
                    Self::deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
                let mut file_entries_to_request =
                    Self::select_files(&mut client, &mut fs_query, &mut cl_query, file_entities);

                Self::remove_duplicates(&mut file_entries_to_release, &mut file_entries_to_request);

                Self::release_file_entries(&mut commands, &mut client, file_entries_to_release);
                Self::request_file_entries(&mut commands, &mut client, file_entries_to_request);

                return Action::SelectEntries(deselected_row_entities);
            }
            Action::NewEntry(
                parent_entity_opt,
                new_file_name,
                entry_kind,
                old_entity_opt,
                entry_contents_opt,
            ) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Res<Global>,
                    ResMut<TabManager>,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
                    Query<&mut FileSystemParent>,
                )> = SystemState::new(world);
                let (
                    mut commands,
                    mut client,
                    global,
                    mut tab_manager,
                    mut fs_query,
                    mut cl_query,
                    mut parent_query,
                ) = system_state.get_mut(world);

                let (deselected_row_entities, file_entries_to_release) =
                    Self::deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
                Self::release_file_entries(&mut commands, &mut client, file_entries_to_release);

                let parent_entity = {
                    if let Some(parent_entity) = parent_entity_opt {
                        *parent_entity
                    } else {
                        global.project_root_entity
                    }
                };

                // expand parent if it's not expanded
                {
                    if let Ok((_, mut fs_ui_state)) = fs_query.get_mut(parent_entity) {
                        fs_ui_state.opened = true;
                    }
                }

                // actually create new entry
                let mut parent = parent_query.get_mut(parent_entity).unwrap();

                let entity_id = self.create_fs_entry(
                    &mut commands,
                    &mut client,
                    &mut parent,
                    parent_entity_opt,
                    new_file_name,
                    entry_kind,
                    entry_contents_opt,
                );

                // migrate undo entities
                if let Some(old_entity) = old_entity_opt {
                    self.migrate_undo_entities(*old_entity, entity_id);
                }

                // open tab for new entry
                tab_manager.open_tab(&mut client, &entity_id);

                system_state.apply(world);

                return Action::DeleteEntry(entity_id, Some(deselected_row_entities));
            }
            Action::DeleteEntry(file_entity, files_to_select_opt) => {
                let mut system_state: SystemState<(
                    Commands,
                    Client,
                    Res<Global>,
                    Query<(Entity, &mut FileSystemUiState)>,
                    Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
                    Query<(&FileSystemEntry, Option<&HasParent>, Option<&NoParent>)>,
                    Query<&mut FileSystemParent>,
                )> = SystemState::new(world);
                let (
                    mut commands,
                    mut client,
                    global,
                    mut ui_query,
                    mut cl_query,
                    fs_query,
                    mut parent_query,
                ) = system_state.get_mut(world);
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
                            let entries = Self::convert_contents_to_slim_tree(
                                &client,
                                file_entity,
                                &fs_query,
                                &mut parent_query,
                            );

                            Some(entries)
                        }
                    }
                };

                // actually delete the entry
                commands.entity(*file_entity).despawn();

                if let Some(files_to_select) = files_to_select_opt {
                    let file_entries_to_request = Self::select_files(
                        &mut client,
                        &mut ui_query,
                        &mut cl_query,
                        files_to_select,
                    );
                    Self::request_file_entries(&mut commands, &mut client, file_entries_to_request);
                }

                system_state.apply(world);

                return Action::NewEntry(
                    parent_entity_opt,
                    entry_name,
                    entry_kind,
                    Some(*file_entity),
                    entry_contents_opt
                        .map(|entries| entries.into_iter().map(|(_, tree)| tree).collect()),
                );
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
        client: &mut Client,
        fs_query: &mut Query<(Entity, &mut FileSystemUiState)>,
        cl_query: &mut Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
        row_entities: &Vec<Entity>,
    ) -> HashSet<Entity> {
        let mut file_entries_to_request = HashSet::new();
        for row_entity in row_entities {
            if let Ok((_, mut ui_state)) = fs_query.get_mut(*row_entity) {
                // File System
                ui_state.selected = true;

                file_entries_to_request.insert(*row_entity);
            }
            if let Ok((_, cl_entry, mut ui_state)) = cl_query.get_mut(*row_entity) {
                // Changelist
                ui_state.selected = true;

                if let Some(file_entity) = cl_entry.file_entity.get(client) {
                    file_entries_to_request.insert(file_entity);
                }
            }
        }
        file_entries_to_request
    }

    fn deselect_all_selected_files(
        client: &mut Client,
        fs_query: &mut Query<(Entity, &mut FileSystemUiState)>,
        cl_query: &mut Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
    ) -> (Vec<Entity>, HashSet<Entity>) {
        let mut deselected_row_entities = Vec::new();
        let mut file_entries_to_release = HashSet::new();
        for (item_entity, mut ui_state) in fs_query.iter_mut() {
            // FileSystem
            if ui_state.selected {
                ui_state.selected = false;

                deselected_row_entities.push(item_entity);
                file_entries_to_release.insert(item_entity);
            }
        }
        for (item_entity, cl_entry, mut ui_state) in cl_query.iter_mut() {
            // Changelist
            if ui_state.selected {
                ui_state.selected = false;

                deselected_row_entities.push(item_entity);

                if let Some(file_entity) = cl_entry.file_entity.get(client) {
                    file_entries_to_release.insert(file_entity);
                }
            }
        }
        (deselected_row_entities, file_entries_to_release)
    }

    fn request_file_entries(
        commands: &mut Commands,
        client: &mut Client,
        file_entries_to_request: HashSet<Entity>,
    ) {
        for file_entity in file_entries_to_request {
            let mut entity_mut = commands.entity(file_entity);
            if entity_mut.authority(client).is_some() {
                entity_mut.request_authority(client);
            }
        }
    }

    fn release_file_entries(
        commands: &mut Commands,
        client: &mut Client,
        file_entries_to_release: HashSet<Entity>,
    ) {
        for file_entity in file_entries_to_release {
            let mut entity_mut = commands.entity(file_entity);
            if entity_mut.authority(client).is_some() {
                entity_mut.release_authority(client);
            }
        }
    }

    fn remove_duplicates(set_a: &mut HashSet<Entity>, set_b: &mut HashSet<Entity>) {
        set_a.retain(|item| {
            if set_b.contains(item) {
                set_b.remove(item);
                false // Remove the item from set_a
            } else {
                true // Keep the item in set_a
            }
        });
    }

    fn create_fs_entry(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        parent: &mut FileSystemParent,
        parent_entity_opt: &Option<Entity>,
        new_file_name: &str,
        entry_kind: &EntryKind,
        entry_contents_opt: &Option<Vec<FileTree>>,
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
            let mut child_component = HasParent::new();
            child_component.parent_id.set(client, &parent_entity);
            commands.entity(entity_id).insert(child_component);
        } else {
            commands.entity(entity_id).insert(NoParent);
        }

        // add UiState component
        file_post_process::insert_ui_state_component(commands, entity_id, true);

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
        fs_query: &Query<(&FileSystemEntry, Option<&HasParent>, Option<&NoParent>)>,
        parent_query: &mut Query<&mut FileSystemParent>,
    ) -> Vec<(Entity, FileTree)> {
        let mut trees = Vec::new();

        if let Ok(parent) = parent_query.get(*parent_entity) {
            let children_entities = parent.get_children();
            for child_entity in children_entities {
                let (child_entry, _, _) = fs_query.get(child_entity).unwrap();
                let slim_tree = FileTree::new(
                    child_entity,
                    child_entry.name.to_string(),
                    *child_entry.kind,
                );
                trees.push((child_entity, slim_tree));
            }

            for (entry_entity, tree) in trees.iter_mut() {
                let subtree = Self::convert_contents_to_slim_tree(
                    client,
                    entry_entity,
                    fs_query,
                    parent_query,
                );
                if subtree.len() > 0 {
                    tree.children = Some(
                        subtree
                            .into_iter()
                            .map(|(_, child_tree)| child_tree)
                            .collect(),
                    );
                }
            }
        }

        trees
    }
    fn migrate_undo_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action in self.undo_actions.iter_mut() {
            action.migrate_entities(old_entity, new_entity);
        }
    }
}
