use std::collections::HashMap;

use bevy_ecs::system::Resource;
use naia_bevy_server::{RoomKey, UserKey};

pub struct UserInfo {
    pub username: String,
    pub room_key: Option<RoomKey>,
}

impl UserInfo {
    pub fn new(username: &str) -> Self {
        Self { username: username.to_string(), room_key: None }
    }

    pub(crate) fn set_room_key(&mut self, room_key: RoomKey) {
        self.room_key = Some(room_key);
    }
}

#[derive(Resource)]
pub struct UserManager {
    credentials: HashMap<String, String>,
    users: HashMap<UserKey, UserInfo>
}

impl Default for UserManager {
    fn default() -> Self {
        let mut credentials = HashMap::new();

        // Connor
        credentials.insert(
            "connorcarpenter".to_string(),
            "greattobealive!".to_string()
        );

        // Brendon?
        credentials.insert(
            "brendoncarpenter".to_string(),
            "greattobealive!".to_string(),
        );

        // TODO: add more users here? get from database?

        Self { credentials, users: HashMap::new() }
    }
}

impl UserManager {
    pub fn validate_user(&self, username: &str, password: &str) -> bool {
        match self.credentials.get(username) {
            Some(p) => p == password,
            None => false,
        }
    }

    pub fn add_user(&mut self, user_key: &UserKey, username: &str) {
        self.users.insert(*user_key, UserInfo::new(username));
    }

    pub fn user_info(&self, user_key: &UserKey) -> Option<&UserInfo> {
        self.users.get(user_key)
    }

    pub fn user_info_mut(&mut self, user_key: &UserKey) -> Option<&mut UserInfo> {
        self.users.get_mut(user_key)
    }
}
