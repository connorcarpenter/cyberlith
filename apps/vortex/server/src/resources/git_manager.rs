use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource, SystemState},
    world::World,
};
use bevy_log::info;
use git2::{Cred, Repository, Tree};

use naia_bevy_server::{CommandsExt, ReplicationConfig, RoomKey, Server, UserKey};

use vortex_proto::{
    components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild},
    messages::ChangelistMessage,
    resources::FileEntryKey,
    types::TabId,
    FileExtension,
};

use crate::{
    components::FileSystemOwner,
    config::GitConfig,
    files::{post_process_networked_entities, FileReadOutput, FileWriter, MeshReader, SkelReader},
    resources::{
        user_manager::UserInfo, workspace::Workspace, ContentEntityData, FileEntryValue,
        UserManager, VertexManager,
    },
};

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
        world: &mut World,
        user_key: UserKey,
        message: ChangelistMessage,
    ) {
        let user_manager = world.get_resource::<UserManager>().unwrap();
        let user_info = user_manager.user_info(&user_key).unwrap();
        let username = user_info.get_username().to_string();
        let email = user_info.get_email().to_string();

        let Some(workspace) = self.workspaces.get_mut(&username) else {
            panic!("Could not find workspace for user: `{}`", username);
        };

        workspace.commit_changelist_entry(world, &user_key, &username, &email, message);
    }

    pub fn rollback_changelist_entry(
        &mut self,
        world: &mut World,
        user_key: UserKey,
        message: ChangelistMessage,
    ) {
        let user_manager = world.get_resource::<UserManager>().unwrap();
        let user_info = user_manager.user_info(&user_key).unwrap();
        let user_name = user_info.get_username().to_string();
        let user_room_key = user_info.get_room_key().unwrap();

        let Some(workspace) = self.workspaces.get_mut(&user_name) else {
            panic!("Could not find workspace for user: `{}`", user_name);
        };

        if let Some((key, value)) = workspace.rollback_changelist_entry(&user_key, world, message) {
            self.spawn_networked_entry_into_world(
                world,
                &user_key,
                &user_name,
                &user_room_key,
                &key,
                &value,
            )
        }
    }

    fn spawn_networked_entry_into_world(
        &mut self,
        world: &mut World,
        user_key: &UserKey,
        user_name: &str,
        user_room_key: &RoomKey,
        entry_key: &FileEntryKey,
        entry_val: &FileEntryValue,
    ) {
        let workspace = self.workspaces.get(user_name).unwrap();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        insert_networked_components_entry(
            &mut commands,
            &mut server,
            user_key,
            user_room_key,
            &workspace.working_file_entries,
            entry_key,
            entry_val,
        );

        system_state.apply(world);
    }

    pub(crate) fn load_content_entities(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        vertex_manager: &mut VertexManager,
        username: &str,
        file_entry_key: &FileEntryKey,
        room_key: &RoomKey,
        tab_id: TabId,
        pause_replication: bool,
    ) -> HashMap<Entity, ContentEntityData> {
        info!("loading content entities");

        let workspace = self.workspaces.get(username).unwrap();
        let output = workspace.load_content_entities(commands, server, file_entry_key);

        let new_entities = match output {
            FileReadOutput::Skel(entities) => {
                SkelReader::post_process_entities(vertex_manager, entities)
            }
            FileReadOutput::Mesh(shape_entities) => {
                MeshReader::post_process_entities(shape_entities)
            }
        };

        post_process_networked_entities(
            commands,
            server,
            room_key,
            &new_entities,
            tab_id,
            pause_replication,
        );

        new_entities
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
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let ext = self.working_file_extension(username, key);
        return ext.write(world, content_entities);
    }

    pub fn working_file_extension(&self, username: &str, key: &FileEntryKey) -> FileExtension {
        self.workspaces
            .get(username)
            .unwrap()
            .working_file_extension(key)
    }

    pub(crate) fn set_changelist_entry_content(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        username: &str,
        key: &FileEntryKey,
        bytes: Box<[u8]>,
    ) {
        let workspace = self.workspaces.get_mut(username).unwrap();

        workspace.set_changelist_entry_content(commands, server, key, bytes);
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

        let mut parent_component = FileSystemChild::new();
        parent_component.parent_id.set(server, &parent_entity);
        commands.entity(entry_entity).insert(parent_component);
    } else {
        commands.entity(entry_entity).insert(FileSystemRootChild);
    }
}
