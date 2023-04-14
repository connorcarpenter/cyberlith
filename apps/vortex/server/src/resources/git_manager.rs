use std::{fs, path::Path};

use bevy_ecs::system::Resource;
use bevy_log::info;
use naia_bevy_server::UserKey;
use git2::{Cred, Repository, ResetType};

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

    pub fn init_dir(&mut self, user_key: &UserKey, user_info: &UserInfo) {

        // Create User's Working directory if it doesn't already exist
        let root_dir = "target/users";
        let user_dir_name = &user_info.username;
        let full_path_str = format!("{}/{}", root_dir, user_dir_name);
        let path = Path::new(&full_path_str);
        let repo_url = self.config.as_ref().unwrap().repo_url.as_str();
        let token = self.config.as_ref().unwrap().access_token.as_str();

        // Initialize Git credentials
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("token", token)
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        if !path.exists() {
            fs::create_dir_all(path).unwrap();

            // Put fetch options into builder
            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fetch_options);

            // Clone repo
            let repo = builder.clone(repo_url, path).unwrap();

            info!("initialized repo at: `{}`", &full_path_str);
        } else {
            info!("repo exists at: `{}`", &full_path_str);

            // Open repo
            let repo = Repository::open(path).unwrap();
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


    }
}


