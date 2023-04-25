use std::{fs, path::Path};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Resource},
};
use bevy_log::info;
use git2::{Cred, Repository, ResetType, Tree};
use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};
use vortex_proto::components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::{components::FileSystemOwner, config::GitConfig, resources::user_manager::UserInfo};

#[derive(Resource)]
pub struct GitManager {
    config: Option<GitConfig>,
}

impl Default for GitManager {
    fn default() -> Self {
        Self { config: None }
    }
}

impl GitManager {
    pub fn use_config(&mut self, config: &GitConfig) {
        self.config = Some(config.clone());
    }

    pub fn init_dir(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        user_key: &UserKey,
        user_info: &UserInfo,
    ) {
        // Create User's Working directory if it doesn't already exist
        let root_dir = "target/users";
        let user_dir_name = &user_info.username;
        let full_path_str = format!("{}/{}", root_dir, user_dir_name);
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

                // Reset local changes
                let head_obj = repo.revparse_single("HEAD").unwrap();
                repo.reset(&head_obj, ResetType::Hard, None).unwrap();

                // Get head of remote branch, merge into local repo
                let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
                let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
                repo.merge(&[&fetch_commit], None, None).unwrap();

                info!("pulled repo with new changes");
            }

            repo
        };

        let head = repo.head().unwrap();
        let tree = head.peel_to_tree().unwrap();
        walk_file_tree(
            commands,
            server,
            &repo,
            &tree,
            user_key,
            user_info.room_key.as_ref().unwrap(),
            None,
        );
    }
}

fn walk_file_tree(
    commands: &mut Commands,
    server: &mut Server,
    repo: &Repository,
    entries: &Tree,
    user_key: &UserKey,
    room_key: &RoomKey,
    parent: Option<Entity>,
) {
    for entry in entries.iter() {
        let name = entry.name().unwrap().to_string();

        match entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let id = spawn_file_system_entry(
                    commands, server, &name, user_key, room_key, &parent, true,
                );

                let children = entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                walk_file_tree(
                    commands,
                    server,
                    repo,
                    &children,
                    user_key,
                    room_key,
                    Some(id),
                );
            }
            Some(git2::ObjectType::Blob) => {
                let _ = spawn_file_system_entry(
                    commands, server, &name, user_key, room_key, &parent, false,
                );
            }
            _ => {}
        }
    }
}

fn spawn_file_system_entry(
    commands: &mut Commands,
    server: &mut Server,
    name: &str,
    user_key: &UserKey,
    room_key: &RoomKey,
    parent: &Option<Entity>,
    is_dir: bool,
) -> Entity {
    let entry_kind = if is_dir {
        EntryKind::Directory
    } else {
        EntryKind::File
    };
    let entity_id = commands
        .spawn_empty()
        .enable_replication(server)
        .insert(FileSystemEntry::new(&name, entry_kind))
        .insert(FileSystemOwner(*user_key))
        .id();

    server.room_mut(room_key).add_entity(&entity_id);

    if let Some(parent) = parent {
        let mut parent_component = FileSystemChild::new();
        parent_component.parent_id.set(server, parent);
        commands.entity(entity_id).insert(parent_component);
    } else {
        commands.entity(entity_id).insert(FileSystemRootChild);
    }

    entity_id
}
