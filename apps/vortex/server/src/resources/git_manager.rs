use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
    world::World,
};
use bevy_log::info;
use git2::{Cred, Repository, Tree};
use naia_bevy_server::{CommandsExt, ReplicationConfig, RoomKey, Server, UserKey};

use vortex_proto::{
    components::{ChangelistEntry, EntryKind, FileSystemEntry, HasParent, NoParent},
    resources::FileEntryKey,
};

use crate::{
    components::FileSystemOwner,
    config::GitConfig,
    files::FileExtension,
    resources::{FileEntryValue, user_manager::UserInfo, workspace::Workspace},
};
use crate::files::FileWriter;

#[derive(Resource)]
pub struct GitManager {
    config: Option<GitConfig>,
    workspaces: HashMap<String, Workspace>,
}

impl Default for GitManager {
    fn default() -> Self {
        Self {
            config: None,
            workspaces: HashMap::new(),
        }
    }
}

impl GitManager {
    pub fn use_config(&mut self, config: &GitConfig) {
        self.config = Some(config.clone());
    }

    pub fn has_workspace(&mut self, user_info: &UserInfo) -> bool {
        self.workspaces.contains_key(user_info.get_username())
    }

    pub fn get_workspace_room_key(&self, user_info: &UserInfo) -> Option<RoomKey> {
        if let Some(workspace) = self.workspaces.get(user_info.get_username()) {
            Some(workspace.room_key.clone())
        } else {
            None
        }
    }

    pub fn workspace_mut(&mut self, username: &str) -> &mut Workspace {
        self.workspaces.get_mut(username).unwrap()
    }

    pub fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("token", access_token)
        });

        remote_callbacks
    }

    pub fn add_workspace(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        user_info: &UserInfo,
    ) {
        // Create User's Working directory if it doesn't already exist
        let username = user_info.get_username();

        let root_dir = "target/users";
        let full_path_str = format!("{}/{}", root_dir, username);
        let path = Path::new(&full_path_str);
        let repo_url = self.config.as_ref().unwrap().repo_url.as_str();
        let access_token = self.config.as_ref().unwrap().access_token.as_str();

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(Self::get_remote_callbacks(access_token));

        let repo = if !path.exists() {
            // Create new directory
            fs::create_dir_all(path).unwrap();

            // Put fetch options into builder
            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fetch_options);

            // Clone repo
            let repo = builder.clone(repo_url, path).unwrap();

            info!("initialized repo at: `{}`", &full_path_str);

            repo
        } else {
            info!("repo exists at: `{}`", &full_path_str);

            // Open repo
            let repo = Repository::open(path).unwrap();

            {
                let mut remote = repo.find_remote("origin").unwrap();
                remote
                    .fetch(&["main"], Some(&mut fetch_options), None)
                    .unwrap();

                let reference = repo.find_reference("FETCH_HEAD").unwrap();
                let target = reference.peel_to_commit().unwrap();

                // Set up a CheckoutBuilder to force the working directory to match the target
                let mut checkout_builder = git2::build::CheckoutBuilder::new();
                checkout_builder.force();

                // Reset local changes
                repo.reset(
                    target.as_object(),
                    git2::ResetType::Hard,
                    Some(&mut checkout_builder),
                )
                .unwrap();

                info!("pulled repo with new changes");
            }

            repo
        };

        let mut file_entries = HashMap::new();

        {
            let head = repo.head().unwrap();
            let tree = head.peel_to_tree().unwrap();

            fill_file_entries_from_git(&mut file_entries, commands, server, &repo, &tree, "", None);
        }

        let new_workspace = Workspace::new(
            user_info.get_room_key().unwrap(),
            file_entries,
            repo,
            access_token,
            &full_path_str,
        );

        insert_networked_components(
            commands,
            server,
            &new_workspace.master_file_entries,
            user_key,
            &new_workspace.room_key,
        );

        self.workspaces.insert(username.to_string(), new_workspace);
    }

    pub fn commit_changelist_entry(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user: &UserInfo,
        commit_message: &str,
        entity: &Entity,
        query: &Query<&ChangelistEntry>,
    ) {
        let username = user.get_username();

        let Some(workspace) = self.workspaces.get_mut(username) else {
            return;
        };

        let email = user.get_email();

        workspace.commit_changelist_entry(
            username,
            email,
            commit_message,
            commands,
            server,
            entity,
            query,
        );
    }

    pub fn rollback_changelist_entry(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        user: &UserInfo,
        entity: &Entity,
        query: &Query<&ChangelistEntry>,
    ) {
        if let Some(workspace) = self.workspaces.get_mut(user.get_username()) {
            if let Some((key, value)) =
                workspace.rollback_changelist_entry(commands, server, entity, query)
            {
                self.spawn_networked_entry_into_world(
                    commands, server, user_key, user, &key, &value,
                )
            }
        }
    }

    pub fn spawn_networked_entry_into_world(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        user_info: &UserInfo,
        entry_key: &FileEntryKey,
        entry_val: &FileEntryValue,
    ) {
        let room_key = user_info.get_room_key().unwrap();
        let workspace = self.workspaces.get(user_info.get_username()).unwrap();
        insert_networked_components_entry(
            commands,
            server,
            user_key,
            &room_key,
            &workspace.working_file_entries,
            entry_key,
            entry_val,
        );
    }

    pub(crate) fn load_content_entities(
        &mut self,
        commands: &mut Commands,
        server: &Server,
        file_entry_key: &FileEntryKey,
        username: &str,
    ) -> Vec<Entity> {
        let workspace = self.workspaces.get(username).unwrap();
        workspace.load_content_entities(commands, server, file_entry_key)
    }

    pub(crate) fn can_read(&self, username: &str, key: &FileEntryKey) -> bool {
        let ext = self.working_file_extension(username, key);
        return ext.can_io();
    }

    pub(crate) fn can_write(&self, username: &str, key: &FileEntryKey) -> bool {
        let ext = self.working_file_extension(username, key);
        return ext.can_io();
    }

    pub(crate) fn write(
        &self,
        username: &str,
        key: &FileEntryKey,
        world: &mut World,
        content_entities: &Vec<Entity>,
    ) -> Box<[u8]> {
        let ext = self.working_file_extension(username, key);
        return ext.write(world, content_entities);
    }

    fn working_file_extension(&self, username: &str, key: &FileEntryKey) -> FileExtension {
        self.workspaces
            .get(username)
            .unwrap()
            .working_file_extension(key)
    }

    pub(crate) fn new_modified_changelist_entry(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        username: &str,
        key: &FileEntryKey,
        bytes: Box<[u8]>,
    ) {
        let workspace = self.workspaces.get_mut(username).unwrap();

        workspace.new_modified_changelist_entry(commands, server, key, bytes);
    }

    pub fn spawn_file_tree_entity(commands: &mut Commands, server: &mut Server) -> Entity {
        let entity_id = commands
            .spawn_empty()
            .enable_replication(server)
            .configure_replication(ReplicationConfig::Delegated)
            .id();

        entity_id
    }
}

fn fill_file_entries_from_git(
    file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
    commands: &mut Commands,
    server: &mut Server,
    repo: &Repository,
    git_tree: &Tree,
    path: &str,
    parent: Option<FileEntryKey>,
) -> HashSet<FileEntryKey> {
    let mut output = HashSet::new();

    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        info!("Git -> Tree: processing Entry `{:?}`", name);

        match git_entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let entry_kind = EntryKind::Directory;
                let id = GitManager::spawn_file_tree_entity(commands, server);

                let file_entry_key = FileEntryKey::new(path, &name, entry_kind);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                let new_path = file_entry_key.path_for_children();
                let children = fill_file_entries_from_git(
                    file_entries,
                    commands,
                    server,
                    repo,
                    &git_children,
                    &new_path,
                    Some(file_entry_key.clone()),
                );

                let file_entry_value =
                    FileEntryValue::new(id, parent.clone(), Some(children), None);
                file_entries.insert(file_entry_key.clone(), file_entry_value);

                output.insert(file_entry_key.clone());
            }
            Some(git2::ObjectType::Blob) => {
                let entry_kind = EntryKind::File;
                let id = GitManager::spawn_file_tree_entity(commands, server);

                let file_entry_key = FileEntryKey::new(path, &name, entry_kind);
                let file_extension = FileExtension::from_file_name(&name);
                let file_entry_value =
                    FileEntryValue::new(id, parent.clone(), None, Some(file_extension));
                file_entries.insert(file_entry_key.clone(), file_entry_value);

                output.insert(file_entry_key.clone());
            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }

    output
}

fn insert_networked_components(
    commands: &mut Commands,
    server: &mut Server,
    file_entries: &HashMap<FileEntryKey, FileEntryValue>,
    user_key: &UserKey,
    room_key: &RoomKey,
) {
    for (file_entry_key, file_entry_value) in file_entries.iter() {
        info!(
            "Networking: walking tree for Entry `{:?}`",
            file_entry_key.name()
        );

        insert_networked_components_entry(
            commands,
            server,
            user_key,
            room_key,
            file_entries,
            file_entry_key,
            file_entry_value,
        );
    }
}

fn insert_networked_components_entry(
    commands: &mut Commands,
    server: &mut Server,
    user_key: &UserKey,
    room_key: &RoomKey,
    file_entries: &HashMap<FileEntryKey, FileEntryValue>,
    entry_key: &FileEntryKey,
    entry_val: &FileEntryValue,
) {
    let entry_entity = entry_val.entity();

    // Insert components
    commands
        .entity(entry_entity)
        .insert(FileSystemEntry::new(&entry_key.name(), entry_key.kind()))
        .insert(FileSystemOwner(*user_key))
        .insert(entry_key.clone());

    // Add entity to room
    server.room_mut(room_key).add_entity(&entry_entity);

    // Add parent entity to component
    if let Some(parent_key) = entry_val.parent() {
        let parent_entity = file_entries.get(parent_key).unwrap().entity();

        let mut parent_component = HasParent::new();
        parent_component.parent_id.set(server, &parent_entity);
        commands.entity(entry_entity).insert(parent_component);
    } else {
        commands.entity(entry_entity).insert(NoParent);
    }
}
