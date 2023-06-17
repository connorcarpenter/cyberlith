use std::collections::HashMap;

use bevy_ecs::system::Resource;
use naia_bevy_server::{RoomKey, UserKey};

pub struct UserInfo {
    username: String,
    email: String,
    workspace_room_key: Option<RoomKey>,
}

impl UserInfo {
    pub fn new(username: &str, email: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            workspace_room_key: None,
        }
    }

    pub(crate) fn get_username(&self) -> &str {
        &self.username
    }

    pub(crate) fn get_email(&self) -> &str {
        &self.email
    }

    pub(crate) fn set_room_key(&mut self, room_key: RoomKey) {
        self.workspace_room_key = Some(room_key);
    }

    pub(crate) fn get_room_key(&self) -> Option<RoomKey> {
        self.workspace_room_key
    }
}

#[derive(Resource)]
pub struct UserManager {
    // HashMap<username, (email, password)>
    credentials: HashMap<String, (String, String)>,
    users: HashMap<UserKey, UserInfo>,
}

impl Default for UserManager {
    fn default() -> Self {
        let mut credentials = HashMap::new();

        // Connor
        credentials.insert(
            "connorcarpenter".to_string(), (
                "connorcarpenter@gmail.com".to_string(),
                "greattobealive!".to_string()
            )
        );

        // Brendon?
        credentials.insert(
            "brendoncarpenter".to_string(), (
                "brendon.e.carpenter@gmail.com".to_string(),
                "greattobealive!".to_string()
            )
        );

        // TODO: add more users here? get from database?

        Self {
            credentials,
            users: HashMap::new(),
        }
    }
}

impl UserManager {
    pub fn validate_user(&self, username: &str, password: &str) -> Option<String> {
        match self.credentials.get(username) {
            Some((email, p)) => {
                if p == password {
                    Some(email.clone())
                } else {
                    None
                }
            },
            None => None,
        }
    }

    pub fn add_user(&mut self, user_key: &UserKey, username: &str, email: &str) {
        self.users.insert(*user_key, UserInfo::new(username, email));
    }

    pub fn user_info(&self, user_key: &UserKey) -> Option<&UserInfo> {
        self.users.get(user_key)
    }

    pub fn user_info_mut(&mut self, user_key: &UserKey) -> Option<&mut UserInfo> {
        self.users.get_mut(user_key)
    }

    pub fn user_name(&self, user_key: &UserKey) -> Option<&str> {
        self.users.get(user_key).map(|u| u.get_username())
    }
}
