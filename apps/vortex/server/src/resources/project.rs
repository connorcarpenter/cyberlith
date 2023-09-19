use std::{collections::HashMap, fs, fs::File, io::Read, path::Path, sync::Mutex};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, SystemState},
    world::World,
};
use bevy_log::{info, warn};
use git2::{Repository, Signature};

use naia_bevy_server::{BigMapKey, CommandsExt, RoomKey, Server, UserKey};

use vortex_proto::{
    components::{ChangelistEntry, ChangelistStatus, EntryKind, FileExtension, FileSystemEntry},
    messages::ChangelistMessage,
    resources::FileKey,
};

use crate::{
    files::{despawn_file_content_entities, load_content_entities, FileWriter},
    resources::{ChangelistValue, ContentEntityData, FileEntryValue, FileSpace, GitManager},
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ProjectKey(u64);

impl BigMapKey for ProjectKey {
    fn to_u64(&self) -> u64 {
        self.0
    }

    fn from_u64(value: u64) -> Self {
        Self(value)
    }
}

pub enum RollbackResult {
    Created,
    Modified(
        FileKey,
        Option<(
            HashMap<Entity, ContentEntityData>,
            HashMap<Entity, ContentEntityData>,
        )>,
    ),
    Deleted(FileKey, FileEntryValue),
}

pub struct Project {
    room_key: RoomKey,
    master_file_entries: HashMap<FileKey, FileEntryValue>,
    working_file_entries: HashMap<FileKey, FileEntryValue>,
    pub changelist_entries: HashMap<FileKey, ChangelistValue>,
    filespaces: HashMap<FileKey, FileSpace>,

    repo: Mutex<Repository>,
    branch: String,
    access_token: String,
    internal_path: String,
}

impl Project {
    pub fn new(
        room_key: RoomKey,
        file_entries: HashMap<FileKey, FileEntryValue>,
        repo: Repository,
        access_token: &str,
        internal_path: &str,
    ) -> Self {
        let working_file_tree = file_entries.clone();
        Self {
            room_key,
            filespaces: HashMap::new(),
            master_file_entries: file_entries,
            working_file_entries: working_file_tree,
            changelist_entries: HashMap::new(),
            repo: Mutex::new(repo),
            access_token: access_token.to_string(),
            branch: "main".to_string(),
            internal_path: internal_path.to_string(),
        }
    }

    pub fn write(
        &self,
        world: &mut World,
        file_key: &FileKey,
        content_entities: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Box<[u8]> {
        let ext = self.working_file_extension(file_key);
        return ext.write(world, self, content_entities);
    }

    pub fn room_key(&self) -> RoomKey {
        self.room_key
    }

    pub(crate) fn file_entity(&self, file_key: &FileKey) -> Option<Entity> {
        let file_entry_val = self.working_file_entries.get(file_key)?;
        Some(file_entry_val.entity())
    }

    pub(crate) fn file_extension(&self, file_key: &FileKey) -> Option<FileExtension> {
        let file_entry_val = self.working_file_entries.get(file_key)?;
        file_entry_val.extension()
    }

    pub fn file_room_key(&self, file_key: &FileKey) -> Option<RoomKey> {
        self.filespaces.get(file_key).map(|fs| fs.room_key())
    }

    pub(crate) fn file_content_entities(
        &self,
        file_key: &FileKey,
    ) -> Option<&HashMap<Entity, ContentEntityData>> {
        self.filespaces
            .get(file_key)
            .map(|fs| fs.content_entities())
    }

    pub(crate) fn has_filespace(&self, file_key: &FileKey) -> bool {
        self.filespaces.contains_key(file_key)
    }

    pub(crate) fn on_insert_content_entity(
        &mut self,
        file_key: &FileKey,
        entity: &Entity,
        content_data: &ContentEntityData,
    ) {
        self.filespaces
            .get_mut(file_key)
            .unwrap()
            .add_content_entity(*entity, content_data.clone());
    }

    pub(crate) fn on_remove_content_entity(
        &mut self,
        server: &mut Server,
        file_key: &FileKey,
        entity: &Entity,
    ) {
        // it's possible the the filespace has already be despawned
        if let Some(filespace) = self.filespaces.get_mut(file_key) {
            filespace.remove_content_entity(entity);
            server.room_mut(&filespace.room_key()).remove_entity(entity);
        }
    }

    pub(crate) fn dependency_file_keys(&self, file_key: &FileKey) -> Vec<FileKey> {
        let mut output = Vec::new();

        // unwrap here doesn't work when file has been closed after being deleted...
        let file_entry_val = self.working_file_entries.get(file_key).unwrap(); // here

        if let Some(dependencies) = file_entry_val.dependencies() {
            for dependency_key in dependencies {
                output.push(dependency_key.clone());

                // perhaps we will need to recurse one day ..
                // output.append(&mut self.dependency_file_keys(dependency_key));
            }
        }
        output
    }

    pub(crate) fn user_join_filespace(
        &mut self,
        world: &mut World,
        user_key: &UserKey,
        file_key: &FileKey,
    ) -> Option<HashMap<Entity, ContentEntityData>> {
        let new_content_entities_opt = if !self.filespaces.contains_key(file_key) {
            let new_entities = self.create_filespace(world, file_key);
            Some(new_entities)
        } else {
            None
        };
        let filespace = self.filespaces.get_mut(file_key).unwrap();
        filespace.user_join(world, user_key);
        new_content_entities_opt
    }

    pub(crate) fn user_leave_filespace(
        &mut self,
        server: &mut Server,
        user_key: &UserKey,
        file_key: &FileKey,
    ) -> Option<HashMap<Entity, ContentEntityData>> {
        let Some(filespace) = self.filespaces.get_mut(file_key) else {
            panic!("Filespace not found");
        };

        filespace.user_leave(server, user_key);
        if filespace.has_no_users() {
            let content_entities = self.delete_filespace(server, file_key);
            return Some(content_entities);
        }

        return None;
    }

    pub fn entity_is_file(&self, entity: &Entity) -> bool {
        Self::find_file_entry_by_entity(&self.working_file_entries, entity).is_some()
    }

    pub fn get_file_key_from_entity(&self, entity: &Entity) -> Option<FileKey> {
        Self::find_file_entry_by_entity(&self.working_file_entries, entity)
    }

    pub fn on_client_create_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        name: &str,
        entity: Entity,
        parent: Option<FileKey>,
        file_key: &FileKey,
    ) {
        if file_key.kind() == EntryKind::Directory {
            info!("creating directory: {}", name);
        } else {
            info!("creating file: {}", name);
        }

        let file_extension = FileExtension::from(name);
        let file_entry_val = FileEntryValue::new(entity, Some(file_extension), parent, None);

        // Add new Entity into Working Tree
        Self::add_to_file_tree(
            &mut self.working_file_entries,
            file_key.clone(),
            file_entry_val,
        );

        // Insert FileEntryKey component to new File Entity
        commands.entity(entity).insert(file_key.clone());

        // New File Entity enters the current project's room
        server.room_mut(&self.room_key()).add_entity(&entity);

        // Update changelist

        if file_key.kind() == EntryKind::File {
            // check whether newly added file already exists in master tree
            let file_exists_in_master = self.master_file_entries.contains_key(&file_key);

            // check whether a changelist entry already exists for this file
            let file_exists_in_changelist = self.changelist_entries.contains_key(&file_key);

            // if file doesn't exist in master tree and no changelist entry exists, then create a changelist entry
            if !file_exists_in_master && !file_exists_in_changelist {
                let default_file_contents_opt = if file_key.kind() == EntryKind::File {
                    Some(file_extension.write_new_default())
                } else {
                    None
                };
                self.new_changelist_entry(
                    commands,
                    server,
                    &file_key,
                    ChangelistStatus::Created,
                    Some(&entity),
                    default_file_contents_opt,
                );
            }

            // if file exists in master tree and a changelist entry exists, then delete the changelist entry
            if file_exists_in_master && file_exists_in_changelist {
                let changelist_entry = self.changelist_entries.remove(&file_key).unwrap();
                commands.entity(changelist_entry.entity()).despawn();
            }
        }
    }

    pub fn on_client_delete_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        changelist_q: &mut Query<&mut ChangelistEntry>,
        entity: &Entity,
    ) {
        // Remove Entity from Working Tree, returning a list of child entities that should be despawned
        let file_key = Self::find_file_entry_by_entity(&self.working_file_entries, entity).unwrap();
        let (_entry_value, entities_to_delete) =
            Self::remove_file_entry(&mut self.working_file_entries, &file_key);

        self.update_changelist_after_despawn(commands, server, changelist_q, &file_key);

        for (child_entity, child_key) in entities_to_delete {
            commands
                .entity(child_entity)
                .take_authority(server)
                .despawn();

            self.update_changelist_after_despawn(commands, server, changelist_q, &child_key);
        }
    }

    pub fn on_client_modify_dir(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        dir_key: &FileKey,
    ) {
        // get children and create changelist entries if needed
        let file_entry_val = self.working_file_entries.get(dir_key).unwrap();
        if let Some(children) = file_entry_val.children() {
            let children: Vec<FileKey> = children.iter().cloned().collect();
            for child_key in children {
                if child_key.kind() == EntryKind::Directory {
                    self.on_client_modify_dir(commands, server, &child_key);
                } else {
                    self.on_client_modify_file(commands, server, &child_key);
                }
            }
        }
    }

    pub fn on_client_modify_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        file_key: &FileKey,
    ) {
        let file_entity = self.file_entity(file_key).unwrap();

        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_key);

        if file_exists_in_changelist {
            // For Modified Changelist entries, if there is no content, then it will be written on Commit
            let entry = self.changelist_entries.get_mut(&file_key).unwrap();
            entry.delete_content();
        } else {
            // create a changelist entry
            self.new_changelist_entry(
                commands,
                server,
                file_key,
                ChangelistStatus::Modified,
                Some(&file_entity),
                None,
            );
        }
    }

    pub fn master_file_entries(&self) -> &HashMap<FileKey, FileEntryValue> {
        &self.master_file_entries
    }

    pub fn working_file_entries(&self) -> &HashMap<FileKey, FileEntryValue> {
        &self.working_file_entries
    }

    pub fn file_add_dependency(&mut self, file_key: &FileKey, dependency_key: &FileKey) {
        info!(
            "file_add_dependency: {:?} -> {:?}",
            file_key, dependency_key
        );
        let file_entry_val = self.working_file_entries.get_mut(file_key).unwrap();
        file_entry_val.add_dependency(dependency_key);
    }

    pub fn file_remove_dependency(&mut self, file_key: &FileKey, dependency_key: &FileKey) {
        info!(
            "file_remove_dependency: {:?} -> {:?}",
            file_key, dependency_key
        );
        let Some(file_entry_val) = self.working_file_entries.get_mut(file_key) else {
            warn!("file_remove_dependency: file_key not found: {:?}", file_key);
            return;
        };
        file_entry_val.remove_dependency(dependency_key);
    }

    fn get_full_file_path_working(
        &self,
        fs_q: &Query<&FileSystemEntry>,
        file_key: &FileKey,
        file_entity: Entity,
    ) -> String {
        let fs_entry = fs_q.get(file_entity).unwrap();
        let file_name = fs_entry.name.as_str();
        let fs_value = self.working_file_entries.get(file_key).unwrap();
        if let Some(parent_file_key) = fs_value.parent() {
            let parent_entity = self
                .working_file_entries
                .get(parent_file_key)
                .unwrap()
                .entity();
            let parent_path = self.get_full_file_path_working(fs_q, parent_file_key, parent_entity);
            format!("{}/{}", parent_path, file_name)
        } else {
            format!("{}", file_name)
        }
    }

    fn get_full_file_path_master(&self, file_key: &FileKey) -> String {
        let file_name = file_key.name();
        let fs_value = self.master_file_entries.get(file_key).unwrap();
        if let Some(parent_file_key) = fs_value.parent() {
            let parent_path = self.get_full_file_path_master(parent_file_key);
            format!("{}/{}", parent_path, file_name)
        } else {
            format!("{}", file_name)
        }
    }

    pub fn commit_changelist_entry(
        &mut self,
        world: &mut World,
        username: &str,
        email: &str,
        message: ChangelistMessage,
    ) {
        let action_status: ChangelistStatus;
        let file_key: FileKey;
        {
            let mut system_state: SystemState<(Server, Query<&ChangelistEntry>)> =
                SystemState::new(world);
            let (server, cl_query) = system_state.get_mut(world);

            let cl_entity: Entity = message.entity.get(&server).unwrap();

            let changelist_entry = cl_query.get(cl_entity).unwrap();
            action_status = *changelist_entry.status;
            file_key = changelist_entry.file_key();

            if file_key.kind() == EntryKind::Directory {
                panic!("should not be able to commit a directory");
            }

            match action_status {
                ChangelistStatus::Modified | ChangelistStatus::Created => {
                    self.changelist_entry_finalize_content(world, &action_status, &file_key);
                }
                ChangelistStatus::Deleted => {}
            }
        }

        let commit_message = message.commit_message.unwrap();

        let mut system_state: SystemState<(Commands, Server, Query<&FileSystemEntry>)> =
            SystemState::new(world);
        let (mut commands, mut server, fs_entry_q) = system_state.get_mut(world);

        match action_status {
            ChangelistStatus::Modified => {
                let file_entry_val = self.working_file_entries.get(&file_key).unwrap().clone();
                let file_entity = file_entry_val.entity();

                info!("git modify file");
                let path = self.get_full_file_path_working(&fs_entry_q, &file_key, file_entity);
                self.fs_create_or_update_file(&file_key, &path);

                // despawn changelist entity
                self.cleanup_changelist_entry(&mut commands, &file_key);

                // remove auth from file entity
                commands.entity(file_entity).take_authority(&mut server);

                // sync to git repo
                self.git_commit(username, email, &commit_message);
                self.git_push();
            }
            ChangelistStatus::Created => {
                let file_entry_val = self.working_file_entries.get(&file_key).unwrap().clone();
                let file_entity = file_entry_val.entity();

                // update master tree with new file entry & parents
                if let Some(parent_key) = file_entry_val.parent() {
                    self.add_parents_to_master_file_tree(parent_key);
                }
                Self::add_to_file_tree(
                    &mut self.master_file_entries,
                    file_key.clone(),
                    file_entry_val.clone(),
                );

                info!("git create file");
                let path = self.get_full_file_path_working(&fs_entry_q, &file_key, file_entity);
                self.fs_create_or_update_file(&file_key, &path);

                // despawn changelist entity
                self.cleanup_changelist_entry(&mut commands, &file_key);

                // remove auth from file entity
                commands.entity(file_entity).take_authority(&mut server);

                // sync to git repo
                self.git_commit(username, email, &commit_message);
                self.git_push();
            }
            ChangelistStatus::Deleted => {
                let path = self.get_full_file_path_master(&file_key);

                // Remove Entity from Master Tree, returning a list of child entities that should be despawned
                let (_entry_value, entities_to_delete) =
                    Self::remove_file_entry(&mut self.master_file_entries, &file_key);
                self.cleanup_changelist_entry(&mut commands, &file_key);

                for (_, child_key) in entities_to_delete {
                    self.cleanup_changelist_entry(&mut commands, &child_key);
                }

                // delete file
                info!("git delete file");
                self.fs_delete_file(&path);

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
        world: &mut World,
        message: ChangelistMessage,
    ) -> RollbackResult {
        let mut system_state: SystemState<(Server, Query<&ChangelistEntry>)> =
            SystemState::new(world);
        let (server, cl_query) = system_state.get_mut(world);

        let cl_entity: Entity = message.entity.get(&server).unwrap();
        let changelist_entry = cl_query.get(cl_entity).unwrap();

        let status = *changelist_entry.status;
        let file_key = changelist_entry.file_key();
        let file_entity = changelist_entry.file_entity.get(&server).unwrap();

        system_state.apply(world);

        match status {
            ChangelistStatus::Created => self.rollback_created_file(world, &file_key),
            ChangelistStatus::Modified => {
                self.rollback_modified_file(world, &file_key, &file_entity)
            }
            ChangelistStatus::Deleted => self.rollback_deleted_file(world, &file_key),
        }
    }

    fn rollback_created_file(&mut self, world: &mut World, file_key: &FileKey) -> RollbackResult {
        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        // Remove Entity from Working Tree, returning a list of child entities that should be despawned
        let (entry_value, entities_to_delete) =
            Self::remove_file_entry(&mut self.working_file_entries, file_key);

        // despawn row entity
        let row_entity = entry_value.entity();
        commands
            .entity(row_entity)
            .take_authority(&mut server)
            .despawn();

        // cleanup changelist entry
        self.cleanup_changelist_entry(&mut commands, file_key);

        // cleanup children
        for (child_row_entity, child_key) in entities_to_delete {
            commands
                .entity(child_row_entity)
                .take_authority(&mut server)
                .despawn();

            self.cleanup_changelist_entry(&mut commands, &child_key);
        }

        system_state.apply(world);

        RollbackResult::Created
    }

    fn rollback_modified_file(
        &mut self,
        world: &mut World,
        file_key: &FileKey,
        file_entity: &Entity,
    ) -> RollbackResult {
        let mut system_state: SystemState<Commands> = SystemState::new(world);
        let mut commands = system_state.get_mut(world);

        // cleanup changelist entry
        self.cleanup_changelist_entry(&mut commands, &file_key);

        system_state.apply(world);

        // respawn content entities within to previous state
        let result = if self.has_filespace(file_key) {
            let (old_entities, new_entities) =
                self.respawn_file_content_entities(world, &file_entity, &file_key);
            Some((old_entities, new_entities))
        } else {
            None
        };

        RollbackResult::Modified(file_key.clone(), result)
    }

    fn rollback_deleted_file(&mut self, world: &mut World, file_key: &FileKey) -> RollbackResult {
        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        let new_entity = GitManager::spawn_file_tree_entity(&mut commands, &mut server);

        let file_entry_value = self.master_file_entries.get_mut(&file_key).unwrap();
        file_entry_value.set_entity(new_entity);
        let file_entry_value = file_entry_value.clone();

        // update working tree with old file entry
        Self::add_to_file_tree(
            &mut self.working_file_entries,
            file_key.clone(),
            file_entry_value.clone(),
        );

        // despawn changelist entity
        self.cleanup_changelist_entry(&mut commands, &file_key);

        system_state.apply(world);

        RollbackResult::Deleted(file_key.clone(), file_entry_value)
    }

    fn respawn_file_content_entities(
        &mut self,
        world: &mut World,
        file_entity: &Entity,
        file_key: &FileKey,
    ) -> (
        HashMap<Entity, ContentEntityData>,
        HashMap<Entity, ContentEntityData>,
    ) {
        let file_extension = self.working_file_extension(file_key);
        let bytes = self.get_bytes_from_cl_or_fs(file_key);
        if !file_extension.can_io() {
            panic!("can't read file: `{:?}`", file_key.name());
        }

        // despawn all previous entities
        let old_content_entities = self.file_content_entities(file_key).unwrap().clone();
        despawn_file_content_entities(world, self, file_key, &old_content_entities);

        // respawn all entities
        let new_content_entities =
            load_content_entities(world, self, &file_extension, file_key, file_entity, bytes);

        let filespace = self.filespaces.get_mut(file_key).unwrap();
        filespace.set_content_entities(new_content_entities.clone());

        (old_content_entities, new_content_entities)
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

    fn fs_create_or_update_file(&mut self, key: &FileKey, path: &str) {
        self.fs_write_file(key, path);
        self.fs_update_index(path);
    }

    fn fs_delete_file(&mut self, file_path: &str) {
        let repo = self.repo.lock().unwrap();

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

    fn fs_write_file(&mut self, key: &FileKey, path: &str) {
        let file_content = self
            .changelist_entries
            .get(&key)
            .unwrap()
            .get_content()
            .unwrap();

        let full_file_path = format!("{}/{}", self.internal_path, path);

        // Create the directory if it doesn't exist
        if let Some(parent) = Path::new(&full_file_path).parent() {
            info!("git creating directories: `{}`", parent.to_str().unwrap());
            fs::create_dir_all(parent).expect("failed to create directories");
        }

        // Write the file with the desired content
        info!("git writing file at: `{}`", full_file_path);
        fs::write(full_file_path, file_content).expect("Failed to write file");
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

    fn cleanup_changelist_entry(&mut self, commands: &mut Commands, file_key: &FileKey) {
        let Some(changelist_value) = self.changelist_entries.remove(file_key) else {
            panic!("Changelist entry not found for file entry key");
        };
        commands.entity(changelist_value.entity()).despawn();
    }

    fn update_changelist_after_despawn(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        changelist_q: &mut Query<&mut ChangelistEntry>,
        file_key: &FileKey,
    ) {
        if file_key.kind() == EntryKind::Directory {
            // deleted directories don't go into changelist
            return;
        }

        // Update changelist

        // check whether newly added file already exists in master tree
        let file_exists_in_master = self.master_file_entries.contains_key(&file_key);

        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_key);

        // if file doesn't exist in master tree and a changelist entry exists, then delete the changelist entry
        if !file_exists_in_master && file_exists_in_changelist {
            let changelist_entry = self.changelist_entries.remove(&file_key).unwrap();
            commands.entity(changelist_entry.entity()).despawn();
        }

        // if file exists in master tree and no changelist entry exists, then create a changelist entry
        if file_exists_in_master && !file_exists_in_changelist {
            self.new_changelist_entry(
                commands,
                server,
                file_key,
                ChangelistStatus::Deleted,
                None,
                None,
            );
        }

        if file_exists_in_master && file_exists_in_changelist {
            let changelist_entity = self.changelist_entries.get_mut(&file_key).unwrap().entity();
            let mut changelist_entry = changelist_q.get_mut(changelist_entity).unwrap();
            if *changelist_entry.status != ChangelistStatus::Deleted {
                *changelist_entry.status = ChangelistStatus::Deleted;
            }
        }
    }

    fn new_changelist_entry(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        file_key: &FileKey,
        changelist_status: ChangelistStatus,
        entity_opt: Option<&Entity>,
        content_opt: Option<Box<[u8]>>,
    ) {
        let mut changelist_entry = ChangelistEntry::new(
            file_key.kind(),
            file_key.name(),
            file_key.path(),
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
            .insert(file_key.clone(), changelist_value);
    }

    fn create_filespace(
        &mut self,
        world: &mut World,
        file_key: &FileKey,
    ) -> HashMap<Entity, ContentEntityData> {
        info!("Creating filespace for file: {:?}", file_key.name());

        // get file contents from either the changelist or the file system
        let bytes = self.get_bytes_from_cl_or_fs(file_key);

        let file_entity = self.file_entity(file_key).unwrap();
        let file_extension = self.working_file_extension(file_key);

        let content_entities_with_data =
            load_content_entities(world, self, &file_extension, file_key, &file_entity, bytes);

        let mut system_state: SystemState<Server> = SystemState::new(world);
        let mut server = system_state.get_mut(world);
        let file_room_key = server.make_room().key();
        let filespace = FileSpace::new(&file_room_key, content_entities_with_data.clone());
        self.filespaces.insert(file_key.clone(), filespace);

        info!("content entities: {:?}", content_entities_with_data);

        content_entities_with_data
    }

    fn get_bytes_from_cl_or_fs(&self, file_key: &FileKey) -> Box<[u8]> {
        if self.changelist_entries.contains_key(file_key) {
            // get contents of file from changelist
            if let Some(content) = self.changelist_entries.get(file_key).unwrap().get_content() {
                info!("getting bytes from changelist");
                Box::from(content)
            } else {
                info!("getting bytes from file 1");
                self.get_file_contents(file_key)
            }
        } else {
            info!("getting bytes from file 2");
            // get contents of file from file system
            self.get_file_contents(file_key)
        }
    }

    fn delete_filespace(
        &mut self,
        server: &mut Server,
        file_key: &FileKey,
    ) -> HashMap<Entity, ContentEntityData> {
        let filespace = self.filespaces.remove(file_key).unwrap();

        let file_room_key = filespace.room_key();

        // delete file room
        server.room_mut(&file_room_key).destroy();

        // return content entities
        filespace.content_entities().clone()
    }

    fn get_file_contents(&self, key: &FileKey) -> Box<[u8]> {
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

    pub(crate) fn working_file_extension(&self, key: &FileKey) -> FileExtension {
        let value = self.working_file_entries.get(key).unwrap();
        value.extension().unwrap()
    }

    pub(crate) fn set_changelist_entry_content(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        key: &FileKey,
        bytes: Box<[u8]>,
    ) {
        // update Changelist entry with new bytes
        if let Some(changelist_entry) = self.changelist_entries.get_mut(key) {
            changelist_entry.set_content(bytes);
        } else {
            // no changelist entry exists, so create a new one with Modified status
            let file_entity = self.file_entity(key).unwrap();

            self.new_changelist_entry(
                commands,
                server,
                key,
                ChangelistStatus::Modified,
                Some(&file_entity),
                Some(bytes),
            );
        }
    }

    fn add_parents_to_master_file_tree(&mut self, parent_key: &FileKey) {
        if self.master_file_entries.contains_key(&parent_key) {
            // no need to add parents
            return;
        };
        if !self.working_file_entries.contains_key(&parent_key) {
            panic!("parent does not exist in Working Tree!");
        }
        // parent exists in working tree, so add it to master tree
        if let Some(grandparent_key) = self.working_file_entries.get(&parent_key).unwrap().parent()
        {
            let grandparent_key = grandparent_key.clone();
            self.add_parents_to_master_file_tree(&grandparent_key);
        }
        Self::add_to_file_tree(
            &mut self.master_file_entries,
            parent_key.clone(),
            self.working_file_entries.get(&parent_key).unwrap().clone(),
        );
    }

    fn add_to_file_tree(
        file_entries: &mut HashMap<FileKey, FileEntryValue>,
        file_key: FileKey,
        file_entry_value: FileEntryValue,
    ) {
        file_entries.insert(file_key.clone(), file_entry_value.clone());

        let Some(parent_key) = file_entry_value.parent() else {
            return;
        };
        let Some(parent_file_tree) = file_entries.get_mut(&parent_key) else {
            panic!("parent does not exist in FileTree!");
        };
        parent_file_tree.add_child(file_key.clone());
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
        file_entries: &HashMap<FileKey, FileEntryValue>,
        entity: &Entity,
    ) -> Option<FileKey> {
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
        file_entries: &mut HashMap<FileKey, FileEntryValue>,
        key: &FileKey,
    ) -> (FileEntryValue, Vec<(Entity, FileKey)>) {
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
        file_entries: &mut HashMap<FileKey, FileEntryValue>,
        key: &FileKey,
        entities: &mut Vec<(Entity, FileKey)>,
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
        status: &ChangelistStatus,
        file_key: &FileKey,
    ) {
        info!(
            "Finalizing content for changelist file `{}` of status: {:?}",
            file_key.name(),
            status
        );
        let extension = self.working_file_extension(file_key);
        let changelist_value = self.changelist_entries.get_mut(&file_key).unwrap();
        if changelist_value.has_content() {
            // changelist entry already has content, backed up last time tab closed
            // nothing left to do here
            info!("Changelist entry already has content, nothing left to do here");
            return;
        } else {
            info!("Changelist entry has no content, meaning some data has been mutated. Need to generate content from entities in world.");

            // get extension and check we can write

            if !extension.can_io() {
                panic!("can't write file: `{:?}`", file_key.name());
            }

            let content_entities: HashMap<Entity, ContentEntityData> = self
                .filespaces
                .get(file_key)
                .unwrap()
                .content_entities()
                .clone();
            let content_entities_opt = Some(content_entities);

            // write
            info!("... Generating content ...");
            let bytes = extension.write(world, self, &content_entities_opt);
            let changelist_value = self.changelist_entries.get_mut(&file_key).unwrap();
            changelist_value.set_content(bytes);
        }
    }
}
