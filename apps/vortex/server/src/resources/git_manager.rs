use std::{fs, path::Path};

use bevy_ecs::system::Resource;
use bevy_log::info;
use naia_bevy_server::UserKey;
use git2::Repository;

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
        let root_dir = "users";
        let user_dir_name = &user_info.username;
        let full_path_str = format!("{}/{}", root_dir, user_dir_name);
        let path = Path::new(&full_path_str);

        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }

        // Use git2 to initialize a new repository in the user's working directory
        let repo = Repository::clone(self.config.as_ref().unwrap().repo_url.as_str(), user_dir_name).unwrap();

        info!("initialized repo in dir: `{}`", user_dir_name);
    }
}


