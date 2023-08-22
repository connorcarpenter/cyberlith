use std::{collections::HashMap, fs, fs::File, io::Read, path::Path, sync::Mutex};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, ResMut, SystemState},
    world::World,
};
use bevy_log::info;
use git2::{Repository, Signature};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use vortex_proto::{
    components::{ChangelistEntry, ChangelistStatus},
    messages::ChangelistMessage,
    resources::FileEntryKey,
    FileExtension,
};

use crate::{
    files::{FileReadOutput, FileReader, FileWriter},
    resources::{
        ShapeWaitlist, ChangelistValue, ContentEntityData, FileEntryValue, GitManager, TabManager, ShapeManager,
    },
};

pub struct Workspace {
    pub room_key: RoomKey,
    pub master_file_entries: HashMap<FileEntryKey, FileEntryValue>,
    pub working_file_entries: HashMap<FileEntryKey, FileEntryValue>,
    pub changelist_entries: HashMap<FileEntryKey, ChangelistValue>,
    repo: Mutex<Repository>,
    access_token: String,
    branch: String,
    internal_path: String,
}

impl Workspace {
    pub fn new(
        room_key: RoomKey,
        file_entries: HashMap<FileEntryKey, FileEntryValue>,
        repo: Repository,
        access_token: &str,
        internal_path: &str,
    ) -> Self {
        let working_file_tree = file_entries.clone();
        Self {
            room_key,
            master_file_entries: file_entries,
            working_file_entries: working_file_tree,
            changelist_entries: HashMap::new(),
            repo: Mutex::new(repo),
            access_token: access_token.to_string(),
            branch: "main".to_string(),
            internal_path: internal_path.to_string(),
        }
    }

    pub fn entity_is_file(&self, entity: &Entity) -> bool {
        Self::find_file_entry_by_entity(&self.working_file_entries, entity).is_some()
    }

    pub fn on_client_create_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        name: &str,
        entity: Entity,
        parent: Option<FileEntryKey>,
        file_entry_key: &FileEntryKey,
    ) {
        info!("creating file: {}", name);

        let file_extension = FileExtension::from_file_name(name);
        let file_entry_val = FileEntryValue::new(entity, parent, None, Some(file_extension));

        // Add new Entity into Working Tree
        Self::add_to_file_tree(
            &mut self.working_file_entries,
            file_entry_key.clone(),
            file_entry_val,
        );

        // Update changelist

        // check whether newly added file already exists in master tree
        let file_exists_in_master = self.master_file_entries.contains_key(&file_entry_key);

        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_entry_key);

        // if file doesn't exist in master tree and no changelist entry exists, then create a changelist entry
        if !file_exists_in_master && !file_exists_in_changelist {
            let default_file_contents = file_extension.write_new_default();
            self.new_changelist_entry(
                commands,
                server,
                &file_entry_key,
                ChangelistStatus::Created,
                Some(&entity),
                Some(default_file_contents),
            );
        }

        // if file exists in master tree and a changelist entry exists, then delete the changelist entry
        if file_exists_in_master && file_exists_in_changelist {
            let changelist_entry = self.changelist_entries.remove(&file_entry_key).unwrap();
            commands.entity(changelist_entry.entity()).despawn();
        }
    }

    pub fn on_client_delete_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        entity: &Entity,
    ) {
        // Remove Entity from Working Tree, returning a list of child entities that should be despawned
        let file_entry_key =
            Self::find_file_entry_by_entity(&self.working_file_entries, entity).unwrap();
        let (_entry_value, entities_to_delete) =
            Self::remove_file_entry(&mut self.working_file_entries, &file_entry_key);

        self.update_changelist_after_despawn(commands, server, &file_entry_key);

        for (child_entity, child_key) in entities_to_delete {
            commands
                .entity(child_entity)
                .take_authority(server)
                .despawn();

            self.update_changelist_after_despawn(commands, server, &child_key);
        }
    }

    pub fn on_client_modify_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        file_entry_key: &FileEntryKey,
        file_entity: &Entity,
    ) {
        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_entry_key);

        if file_exists_in_changelist {
            // For Modified Changelist entries, if there is no content, then it will be written on Commit
            let entry = self.changelist_entries.get_mut(&file_entry_key).unwrap();
            entry.delete_content();
        } else {
            // create a changelist entry
            self.new_changelist_entry(
                commands,
                server,
                file_entry_key,
                ChangelistStatus::Modified,
                Some(file_entity),
                None,
            );
        }
    }

    pub fn commit_entire_changelist(
        &mut self,
        _commands: &mut Commands,
        _server: &Server,
        _query: &Query<&ChangelistEntry>,
    ) {
        todo!();
    }

    pub fn commit_changelist_entry(
        &mut self,
        world: &mut World,
        user_key: &UserKey,
        username: &str,
        email: &str,
        message: ChangelistMessage,
    ) {
        let action_status: ChangelistStatus;
        let file_entry_key: FileEntryKey;
        {
            let mut system_state: SystemState<(Server, Query<&ChangelistEntry>)> =
                SystemState::new(world);
            let (server, cl_query) = system_state.get_mut(world);

            let cl_entity: Entity = message.entity.get(&server).unwrap();

            let changelist_entry = cl_query.get(cl_entity).unwrap();
            action_status = *changelist_entry.status;
            file_entry_key = changelist_entry.file_entry_key();

            match action_status {
                ChangelistStatus::Modified | ChangelistStatus::Created => {
                    self.changelist_entry_finalize_content(
                        world,
                        user_key,
                        &action_status,
                        &file_entry_key,
                    );
                }
                ChangelistStatus::Deleted => {}
            }
        }

        let commit_message = message.commit_message.unwrap();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        match action_status {
            ChangelistStatus::Modified => {
                let file_entry_val = self
                    .working_file_entries
                    .get(&file_entry_key)
                    .unwrap()
                    .clone();
                let file_entity = file_entry_val.entity();

                info!("git modify file");
                self.fs_create_or_update_file(&file_entry_key);

                // despawn changelist entity
                self.cleanup_changelist_entry(&mut commands, &file_entry_key);

                // remove auth from file entity
                commands.entity(file_entity).take_authority(&mut server);

                // sync to git repo
                self.git_commit(username, email, &commit_message);
                self.git_push();
            }
            ChangelistStatus::Created => {
                let file_entry_val = self
                    .working_file_entries
                    .get(&file_entry_key)
                    .unwrap()
                    .clone();
                let file_entity = file_entry_val.entity();

                // update master tree with new file entry
                Self::add_to_file_tree(
                    &mut self.master_file_entries,
                    file_entry_key.clone(),
                    file_entry_val.clone(),
                );

                info!("git create file");
                self.fs_create_or_update_file(&file_entry_key);

                // despawn changelist entity
                self.cleanup_changelist_entry(&mut commands, &file_entry_key);

                // remove auth from file entity
                commands.entity(file_entity).take_authority(&mut server);

                // sync to git repo
                self.git_commit(username, email, &commit_message);
                self.git_push();
            }
            ChangelistStatus::Deleted => {
                // Remove Entity from Master Tree, returning a list of child entities that should be despawned
                let (_entry_value, entities_to_delete) =
                    Self::remove_file_entry(&mut self.master_file_entries, &file_entry_key);
                self.cleanup_changelist_entry(&mut commands, &file_entry_key);

                for (_, child_key) in entities_to_delete {
                    self.cleanup_changelist_entry(&mut commands, &child_key);
                }

                // delete file
                self.fs_delete_file(file_entry_key);

                // sync to git repo
                self.git_commit(username, email, &commit_message);
                self.git_push();
            }
        }

        system_state.apply(world);
    }

    // returns an entity to spawn if delete was rolled back
    pub fn rollback_changelist_entry(
        &mut self,
        user_key: &UserKey,
        world: &mut World,
        message: ChangelistMessage,
    ) -> Option<(FileEntryKey, FileEntryValue)> {
        let mut system_state: SystemState<(
            Commands,
            Server,
            ResMut<TabManager>,
            ResMut<ShapeWaitlist>,
            ResMut<ShapeManager>,
            Query<&ChangelistEntry>,
        )> = SystemState::new(world);
        let (mut commands, mut server, mut tab_manager, mut vertex_waitlist, mut shape_manager, cl_query) =
            system_state.get_mut(world);

        let cl_entity: Entity = message.entity.get(&server).unwrap();
        let changelist_entry = cl_query.get(cl_entity).unwrap();

        let status = *changelist_entry.status;
        let file_entry_key = changelist_entry.file_entry_key();

        let output = match status {
            ChangelistStatus::Modified => {
                let file_entity = changelist_entry.file_entity.get(&server).unwrap();

                // cleanup changelist entry
                self.cleanup_changelist_entry(&mut commands, &file_entry_key);

                // if tab is open, respawn content entities within to previous state
                tab_manager.respawn_tab_content_entities(
                    &mut commands,
                    &mut server,
                    self,
                    &mut vertex_waitlist,
                    &mut shape_manager,
                    user_key,
                    &file_entity,
                    &file_entry_key,
                );

                None
            }
            ChangelistStatus::Created => {
                // Remove Entity from Working Tree, returning a list of child entities that should be despawned
                let (entry_value, entities_to_delete) =
                    Self::remove_file_entry(&mut self.working_file_entries, &file_entry_key);

                // despawn row entity
                let row_entity = entry_value.entity();
                commands
                    .entity(row_entity)
                    .take_authority(&mut server)
                    .despawn();

                // cleanup changelist entry
                self.cleanup_changelist_entry(&mut commands, &file_entry_key);

                // cleanup children
                for (child_row_entity, child_key) in entities_to_delete {
                    commands
                        .entity(child_row_entity)
                        .take_authority(&mut server)
                        .despawn();

                    self.cleanup_changelist_entry(&mut commands, &child_key);
                }

                None
            }
            ChangelistStatus::Deleted => {
                let new_entity = GitManager::spawn_file_tree_entity(&mut commands, &mut server);

                let file_entry_value = self.master_file_entries.get_mut(&file_entry_key).unwrap();
                file_entry_value.set_entity(new_entity);
                let file_entry_value = file_entry_value.clone();

                // update working tree with old file entry
                Self::add_to_file_tree(
                    &mut self.working_file_entries,
                    file_entry_key.clone(),
                    file_entry_value.clone(),
                );

                // despawn changelist entity
                self.cleanup_changelist_entry(&mut commands, &file_entry_key);

                Some((file_entry_key.clone(), file_entry_value))
            }
        };

        system_state.apply(world);

        output
    }

    fn fs_update_index(&mut self, path: &str) {
        let repo = self.repo.lock().unwrap();

        // Add the file to the repository
        let mut index = repo.index().expect("Failed to open index");
        index
            .add_path(Path::new(path))
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");
    }

    fn fs_create_or_update_file(&mut self, key: &FileEntryKey) {
        let file_path = self.fs_write_file(key);
        self.fs_update_index(&file_path);
    }

    fn fs_delete_file(&mut self, key: FileEntryKey) {
        let repo = self.repo.lock().unwrap();

        let file_path = format!("{}{}", key.path(), key.name());
        let full_path = format!("{}/{}", self.internal_path, file_path);
        info!("git deleting file at: `{}`", full_path);

        // Remove the file from the working directory
        fs::remove_file(&full_path).expect("Failed to delete file");

        // Remove the file from the repository index
        let mut index = repo.index().expect("Failed to open index");
        index
            .remove_path(Path::new(&file_path))
            .expect("Failed to remove file from index");
        index.write().expect("Failed to write index");
    }

    fn fs_write_file(&mut self, key: &FileEntryKey) -> String {
        let file_path = format!("{}{}", key.path(), key.name());
        let full_path = format!("{}/{}", self.internal_path, file_path);
        info!("git writing file at: `{}`", full_path);

        let file_content = self
            .changelist_entries
            .get(&key)
            .unwrap()
            .get_content()
            .unwrap();

        // Write the file with the desired content
        fs::write(&full_path, file_content).expect("Failed to write file");

        file_path
    }

    fn git_commit(&mut self, username: &str, email: &str, commit_message: &str) {
        let repo = self.repo.lock().unwrap();

        // get index
        let mut index = repo.index().expect("Failed to open index");

        // Get the updated tree
        let tree_id = index.write_tree().expect("Failed to write tree");

        // Get the current HEAD reference
        let head_reference = repo.head().expect("Failed to get HEAD reference");

        // Get the commit that HEAD points to
        let parent_commit = head_reference
            .peel_to_commit()
            .expect("Failed to peel HEAD to commit");

        // Prepare the commit details
        let author = Signature::now(username, email).expect("Failed to create author signature");
        let committer =
            Signature::now(username, email).expect("Failed to create committer signature");

        // Create the commit
        repo.commit(
            Some("HEAD"),
            &author,
            &committer,
            commit_message,
            &repo.find_tree(tree_id).expect("Failed to find tree"),
            &[&parent_commit],
        )
        .expect("Failed to create commit");
    }

    fn git_push(&self) {
        let repo = self.repo.lock().unwrap();
        let mut remote = repo
            .find_remote("origin")
            .expect("Failed to find remote 'origin'");
        let mut options = git2::PushOptions::new();
        options.remote_callbacks(GitManager::get_remote_callbacks(&self.access_token)); // Set up remote callbacks if needed
        remote
            .push(&[format!("refs/heads/{}", self.branch)], Some(&mut options))
            .expect("Failed to push commit");
    }

    fn cleanup_changelist_entry(&mut self, commands: &mut Commands, file_entry_key: &FileEntryKey) {
        let Some(changelist_value) = self.changelist_entries.remove(file_entry_key) else {
            panic!("Changelist entry not found for file entry key");
        };
        commands.entity(changelist_value.entity()).despawn();
    }

    fn update_changelist_after_despawn(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        file_entry_key: &FileEntryKey,
    ) {
        // Update changelist

        // check whether newly added file already exists in master tree
        let file_exists_in_master = self.master_file_entries.contains_key(&file_entry_key);

        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_entry_key);

        // if file doesn't exist in master tree and a changelist entry exists, then delete the changelist entry
        if !file_exists_in_master && file_exists_in_changelist {
            let changelist_entry = self.changelist_entries.remove(&file_entry_key).unwrap();
            commands.entity(changelist_entry.entity()).despawn();
        }

        // if file exists in master tree and no changelist entry exists, then create a changelist entry
        if file_exists_in_master && !file_exists_in_changelist {
            self.new_changelist_entry(
                commands,
                server,
                file_entry_key,
                ChangelistStatus::Deleted,
                None,
                None,
            );
        }
    }

    fn new_changelist_entry(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        file_entry_key: &FileEntryKey,
        changelist_status: ChangelistStatus,
        entity_opt: Option<&Entity>,
        content_opt: Option<Box<[u8]>>,
    ) {
        let mut changelist_entry = ChangelistEntry::new(
            file_entry_key.kind(),
            file_entry_key.name(),
            file_entry_key.path(),
            changelist_status,
        );
        if let Some(entity) = entity_opt {
            changelist_entry.file_entity.set(server, &entity);
        }

        let changelist_entity = commands
            .spawn_empty()
            .enable_replication(server)
            .insert(changelist_entry)
            .id();

        // Add entity to room
        server
            .room_mut(&self.room_key)
            .add_entity(&changelist_entity);

        let mut changelist_value = ChangelistValue::new(changelist_entity);
        if let Some(content) = content_opt {
            changelist_value.set_content(content);
        }
        self.changelist_entries
            .insert(file_entry_key.clone(), changelist_value);
    }

    pub(crate) fn load_content_entities(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        key: &FileEntryKey,
    ) -> FileReadOutput {
        // get file extension of file
        let file_extension = self.working_file_extension(key);

        // get file contents from either the changelist or the file system
        let bytes = {
            if self.changelist_entries.contains_key(key) {
                // get contents of file from changelist
                if let Some(content) = self.changelist_entries.get(key).unwrap().get_content() {
                    Box::from(content)
                } else {
                    self.get_file_contents(key)
                }
            } else {
                // get contents of file from file system
                self.get_file_contents(key)
            }
        };

        // FileReader reads File's contents and spawns all Entities + Components
        let read_output = file_extension.read(commands, server, &bytes);

        read_output
    }

    fn get_file_contents(&self, key: &FileEntryKey) -> Box<[u8]> {
        let file_path = format!("{}{}", key.path(), key.name());
        let full_path = format!("{}/{}", self.internal_path, file_path);
        info!("Getting blob for file: {}", full_path);
        let path = Path::new(full_path.as_str());
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => panic!("Failed to open file: {}", err),
        };

        let mut contents = Vec::new();
        match file.read_to_end(&mut contents) {
            Ok(_) => Box::from(contents),
            Err(err) => panic!("Failed to read file: {}", err),
        }
    }

    pub(crate) fn working_file_extension(&self, key: &FileEntryKey) -> FileExtension {
        let value = self.working_file_entries.get(key).unwrap();
        value.extension().unwrap()
    }

    pub(crate) fn set_changelist_entry_content(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        key: &FileEntryKey,
        bytes: Box<[u8]>,
    ) {
        // update Changelist entry with new bytes
        if let Some(changelist_entry) = self.changelist_entries.get_mut(key) {
            changelist_entry.set_content(bytes);
        } else {
            // no changelist entry exists, so create a new one with Modified status
            self.new_changelist_entry(
                commands,
                server,
                key,
                ChangelistStatus::Modified,
                None,
                Some(bytes),
            );
        }
    }

    fn add_to_file_tree(
        file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
        file_entry_key: FileEntryKey,
        file_entry_value: FileEntryValue,
    ) {
        file_entries.insert(file_entry_key.clone(), file_entry_value.clone());

        let Some(parent_key) = file_entry_value.parent() else {
            return;
        };
        let Some(parent_file_tree) = file_entries.get_mut(&parent_key) else {
            panic!("parent does not exist in FileTree!");
        };
        parent_file_tree.add_child(file_entry_key.clone());
        info!("Added child to parent entry");
    }

    // fn find_file_tree_mut_by_entity<'a>(file_entries: &'a mut HashMap<FileEntryKey, FileEntryValue>, entity: &Entity) -> Option<&'a mut FileTree> {
    //     for child in file_entries {
    //         if child.entity == *entity {
    //             return Some(child);
    //         }
    //         if let Some(children) = &mut child.children {
    //             let found_child = Self::find_file_tree_mut_by_entity(children, entity);
    //             if found_child.is_some() {
    //                 return found_child;
    //             }
    //         }
    //     }
    //     return None;
    // }
    //

    fn find_file_entry_by_entity(
        file_entries: &HashMap<FileEntryKey, FileEntryValue>,
        entity: &Entity,
    ) -> Option<FileEntryKey> {
        let mut key_opt = None;
        for (entry_key, entry_val) in file_entries.iter() {
            if entry_val.entity() == *entity {
                key_opt = Some(entry_key.clone());
                break;
            }
        }
        if key_opt.is_none() {
            return None;
        }
        let key = key_opt.unwrap();

        Some(key)
    }

    fn remove_file_entry(
        file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
        key: &FileEntryKey,
    ) -> (FileEntryValue, Vec<(Entity, FileEntryKey)>) {
        let mut entities = Vec::new();

        // remove entry
        let removed_entry =
            Self::remove_entry_and_collect_children_entities(file_entries, key, &mut entities);

        // remove entry from parent's children
        if let Some(parent_key) = removed_entry.parent() {
            if let Some(parent) = file_entries.get_mut(&parent_key) {
                parent.remove_child(key);
            }
        }

        return (removed_entry, entities);
    }

    fn remove_entry_and_collect_children_entities(
        file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
        key: &FileEntryKey,
        entities: &mut Vec<(Entity, FileEntryKey)>,
    ) -> FileEntryValue {
        // remove entry
        let removed_entry = file_entries.remove(key).unwrap();

        // handle children
        if let Some(removed_entry_children) = removed_entry.children() {
            for child_key in removed_entry_children {
                let removed_entry = Self::remove_entry_and_collect_children_entities(
                    file_entries,
                    child_key,
                    entities,
                );
                entities.push((removed_entry.entity(), child_key.clone()));
            }
        }

        removed_entry
    }

    fn changelist_entry_finalize_content(
        &mut self,
        world: &mut World,
        user_key: &UserKey,
        status: &ChangelistStatus,
        file_entry_key: &FileEntryKey,
    ) {
        info!(
            "Finalizing content for changelist file `{}` of status: {:?}",
            file_entry_key.name(),
            status
        );
        let extension = self.working_file_extension(file_entry_key);
        let changelist_value = self.changelist_entries.get_mut(&file_entry_key).unwrap();
        if changelist_value.has_content() {
            // changelist entry already has content, backed up last time tab closed
            // nothing left to do here
            info!("Changelist entry already has content, nothing left to do here");
            return;
        } else {
            info!("Changelist entry has no content, meaning some data has been mutated. Need to generate content from entities in world.");

            // get extension and check we can write

            if !extension.can_io() {
                panic!("can't write file: `{:?}`", file_entry_key.name());
            }

            // get entities from TabManager's current tab
            let content_entities: HashMap<Entity, ContentEntityData> = world
                .get_resource::<TabManager>()
                .unwrap()
                .user_current_tab_content_entities(user_key)
                .iter()
                .map(|(entity, data)| (*entity, data.clone()))
                .collect();

            // write
            info!("... Generating content ...");
            let bytes = extension.write(world, &content_entities);
            changelist_value.set_content(bytes);
        }
    }
}
