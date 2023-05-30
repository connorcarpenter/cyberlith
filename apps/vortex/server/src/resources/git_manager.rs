use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource},
};
use bevy_log::info;
use git2::{Cred, Repository, Tree};
use naia_bevy_server::{CommandsExt, ReplicationConfig, RoomKey, Server, UserKey};
use std::{collections::HashMap, fs, path::Path};
use vortex_proto::components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};
use vortex_proto::resources::FileTree;

use crate::{
    components::FileSystemOwner,
    config::GitConfig,
    resources::{user_manager::UserInfo, workspace::Workspace},
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
        let token = self.config.as_ref().unwrap().access_token.clone();

        // Initialize Git credentials
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("token", &token)
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

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

        let head = repo.head().unwrap();
        let tree = head.peel_to_tree().unwrap();

        let file_tree = get_file_tree(commands, server, &repo, &tree);

        let new_workspace = Workspace::new(user_info.get_room_key().unwrap(), file_tree);

        insert_networked_components(
            commands,
            server,
            &new_workspace.file_tree,
            user_key,
            &new_workspace.room_key,
            None,
        );

        self.workspaces.insert(username.to_string(), new_workspace);
    }
}

fn get_file_tree(
    commands: &mut Commands,
    server: &mut Server,
    repo: &Repository,
    git_tree: &Tree,
) -> Vec<FileTree> {
    let mut output = Vec::new();
    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        info!("Git -> Tree: processing Entry `{:?}`", name);

        match git_entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let entry_kind = EntryKind::Directory;
                let id = spawn_file_tree_entity(commands, server);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                let children = get_file_tree(commands, server, repo, &git_children);

                let mut file_tree = FileTree::new(id, name, entry_kind);
                file_tree.children = Some(children);
                output.push(file_tree);
            }
            Some(git2::ObjectType::Blob) => {
                let entry_kind = EntryKind::File;
                let id = spawn_file_tree_entity(commands, server);

                let file_tree = FileTree::new(id, name, entry_kind);
                output.push(file_tree);
            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }
    output
}

fn spawn_file_tree_entity(commands: &mut Commands, server: &mut Server) -> Entity {
    let entity_id = commands
        .spawn_empty()
        .enable_replication(server)
        .configure_replication(ReplicationConfig::Delegated)
        //.insert(FileSystemEntry::new(&name, entry_kind))
        //.insert(FileSystemOwner(*user_key))
        .id();

    //server.room_mut(room_key).add_entity(&entity_id);

    // if let Some(parent) = parent {
    //     let mut parent_component = FileSystemChild::new();
    //     parent_component.parent_id.set(server, parent);
    //     commands.entity(entity_id).insert(parent_component);
    // } else {
    //     commands.entity(entity_id).insert(FileSystemRootChild);
    // }

    entity_id
}

fn insert_networked_components(
    commands: &mut Commands,
    server: &mut Server,
    file_trees: &Vec<FileTree>,
    user_key: &UserKey,
    room_key: &RoomKey,
    parent: Option<Entity>,
) {
    for file_tree in file_trees.iter() {

        info!("Networking: walking tree for Entry `{:?}`", file_tree.name);

        match file_tree.kind {
            EntryKind::Directory => {
                insert_networked_components_entry(
                    commands, server, user_key, room_key, &parent, file_tree,
                );

                insert_networked_components(
                    commands,
                    server,
                    file_tree.children.as_ref().unwrap(),
                    user_key,
                    room_key,
                    Some(file_tree.entity),
                );
            }
            EntryKind::File => {
                insert_networked_components_entry(
                    commands, server, user_key, room_key, &parent, file_tree,
                );
            }
        }
    }
}

fn insert_networked_components_entry(
    commands: &mut Commands,
    server: &mut Server,
    user_key: &UserKey,
    room_key: &RoomKey,
    parent: &Option<Entity>,
    entry: &FileTree,
) {
    let entry_entitiy = entry.entity;

    commands.entity(entry_entitiy)
        .insert(FileSystemEntry::new(&entry.name, entry.kind))
        .insert(FileSystemOwner(*user_key));

    server.room_mut(room_key).add_entity(&entry_entitiy);

    if let Some(parent) = parent {
        let mut parent_component = FileSystemChild::new();
        parent_component.parent_id.set(server, parent);
        commands.entity(entry_entitiy).insert(parent_component);
    } else {
        commands.entity(entry_entitiy).insert(FileSystemRootChild);
    }
}
