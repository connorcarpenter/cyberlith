use std::{fs, path::Path};

use bevy_ecs::system::{Commands, Resource};
use bevy_log::info;
use naia_bevy_server::UserKey;
use git2::{Cred, Repository, ResetType, Tree};

use crate::{config::GitConfig, resources::user_manager::UserInfo};

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

    pub fn init_dir(&mut self, commands: &mut Commands, user_key: &UserKey, user_info: &UserInfo) {

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
                remote.fetch(&["main"], Some(&mut fetch_options), None).unwrap();

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

        self.convert_repo_to_ecs(commands, user_key, &repo);
    }

    fn convert_repo_to_ecs(&mut self, commands: &mut Commands, user_key: &UserKey, repo: &Repository) {
        let head = repo.head().unwrap();
        let tree = head.peel_to_tree().unwrap();
        print_tree(repo, &tree, 0);
    }
}

fn print_tree(repo: &Repository, root: &Tree, depth: u32) {
    for entry in root.iter() {
        let name = entry.name().unwrap().to_string();
        //let entry_path = path.join(&name);

        match entry.kind() {
            Some(git2::ObjectType::Tree) => {
                println!("{:indent$}{}", "", name, indent = depth as usize);
                let tree = entry.to_object(repo).unwrap().peel_to_tree().unwrap();
                print_tree(repo, &tree, depth + 1);
            }
            Some(git2::ObjectType::Blob) => {
                println!("{:indent$}{}", "", name, indent = depth as usize);
            }
            _ => {}
        }
    }
}