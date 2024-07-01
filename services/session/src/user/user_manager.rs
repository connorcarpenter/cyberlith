use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use auth_server_types::UserId;

#[derive(Resource)]
pub struct UserManager {
    login_tokens: HashMap<String, UserData>,
    users: HashMap<UserKey, UserData>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            login_tokens: HashMap::new(),
            users: HashMap::new(),
        }
    }

    // Client login

    pub fn add_login_token(&mut self, user_id: &UserId, token: &str) {
        self.login_tokens
            .insert(token.to_string(), UserData::new(*user_id));
    }

    pub fn take_login_token(&mut self, token: &str) -> Option<UserData> {
        self.login_tokens.remove(token)
    }

    pub fn add_user(&mut self, user_key: UserKey, user_data: UserData) {
        self.users.insert(user_key, user_data);
    }

    pub fn get_user_data(&self, user_key: &UserKey) -> Option<&UserData> {
        self.users.get(user_key)
    }

    pub fn get_user_data_mut(&mut self, user_key: &UserKey) -> Option<&mut UserData> {
        self.users.get_mut(user_key)
    }

    // World Connection

    pub fn get_users_ready_to_connect_to_world(
        &mut self,
        world_connect_resend_rate: &Duration,
    ) -> Vec<(UserKey, UserId)> {
        let now = Instant::now();

        let mut worldless_users = Vec::new();
        for (user_key, user_data) in self.users.iter_mut() {
            if user_data.is_world_connected() {
                continue;
            }
            if !user_data.ready_for_world_connect() {
                continue;
            }
            if let Some(last_sent) = user_data.world_connect_last_sent_to_region {
                let time_since_last_sent = now.duration_since(last_sent);
                if time_since_last_sent >= *world_connect_resend_rate {
                    worldless_users.push((*user_key, user_data.user_id));
                    user_data.world_connect_last_sent_to_region = Some(now);
                }
            } else {
                worldless_users.push((*user_key, user_data.user_id));
                user_data.world_connect_last_sent_to_region = Some(now);
            }
        }
        worldless_users
    }

    pub fn user_set_world_connected(&mut self, user_key: &UserKey, world_instance_secret: &str) {
        let user_data = self.users.get_mut(user_key).unwrap();
        user_data.set_world_connected(world_instance_secret);
    }
}

pub(crate) struct UserData {
    pub(crate) user_id: UserId,

    pub(crate) world_connect_last_sent_to_region: Option<Instant>,
    ready_for_world_connect: bool,

    // LATER this may be used to send meaningful data about a user back to the given world server instance..
    world_instance_secret: Option<String>,
}

impl UserData {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,

            world_connect_last_sent_to_region: None,
            ready_for_world_connect: false,

            world_instance_secret: None, // tells us whether user is connected
        }
    }

    pub fn ready_for_world_connect(&self) -> bool {
        self.ready_for_world_connect
    }

    pub fn make_ready_for_world_connect(&mut self) {
        self.ready_for_world_connect = true;
    }

    pub fn is_world_connected(&self) -> bool {
        self.world_instance_secret.is_some()
    }

    pub fn set_world_connected(&mut self, world_instance_secret: &str) {
        self.world_instance_secret = Some(world_instance_secret.to_string());
    }

    pub fn set_world_disconnected(&mut self) {
        self.world_instance_secret = None;
    }
}
