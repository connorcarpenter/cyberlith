use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use auth_server_types::UserId;
use bevy_http_client::ResponseKey as ClientResponseKey;

use region_server_http_proto::WorldConnectResponse;

pub(crate) struct UserData {
    user_id: UserId,

    last_sent: Option<Instant>,
    ready_for_world_connect: bool,

    // LATER this may be used to send meaningful data about a user back to the given world server instance..
    world_instance_secret: Option<String>,
}

impl UserData {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,

            last_sent: None,
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

struct WorldInstanceData {
    user_id_to_key_map: HashMap<UserId, UserKey>,
}

impl WorldInstanceData {
    pub fn new() -> Self {
        Self {
            user_id_to_key_map: HashMap::new(),
        }
    }

    pub(crate) fn add_user(&mut self, user_key: UserKey, user_id: UserId) {
        self.user_id_to_key_map.insert(user_id, user_key);
    }
}

#[derive(Resource)]
pub struct Global {
    instance_secret: String,

    world_connect_response_keys: HashMap<ClientResponseKey<WorldConnectResponse>, UserKey>,
    world_connect_resend_rate: Duration,

    login_tokens: HashMap<String, UserData>,
    users: HashMap<UserKey, UserData>,
    world_instances: HashMap<String, WorldInstanceData>,
    asset_server_opt: Option<(String, u16)>,
    social_server_opt: Option<(String, u16)>,
}

impl Global {
    pub fn new(
        instance_secret: &str,
        world_connect_resend_rate: Duration,
    ) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
            world_connect_response_keys: HashMap::new(),
            world_connect_resend_rate,
            login_tokens: HashMap::new(),
            users: HashMap::new(),
            world_instances: HashMap::new(),
            asset_server_opt: None,
            social_server_opt: None,
        }
    }

    pub fn instance_secret(&self) -> &str {
        &self.instance_secret
    }

    // World Keys

    pub fn get_users_ready_to_connect_to_world(&mut self) -> Vec<(UserKey, UserId)> {
        let now = Instant::now();

        let mut worldless_users = Vec::new();
        for (user_key, user_data) in self.users.iter_mut() {
            if user_data.is_world_connected() {
                continue;
            }
            if !user_data.ready_for_world_connect() {
                continue;
            }
            if let Some(last_sent) = user_data.last_sent {
                let time_since_last_sent = now.duration_since(last_sent);
                if time_since_last_sent >= self.world_connect_resend_rate {
                    worldless_users.push((*user_key, user_data.user_id));
                    user_data.last_sent = Some(now);
                }
            } else {
                worldless_users.push((*user_key, user_data.user_id));
                user_data.last_sent = Some(now);
            }
        }
        worldless_users
    }

    pub fn add_world_connect_response_key(
        &mut self,
        user_key: &UserKey,
        response_key: ClientResponseKey<WorldConnectResponse>,
    ) {
        self.world_connect_response_keys
            .insert(response_key, user_key.clone());
    }

    pub fn remove_world_connect_response_key(
        &mut self,
        response_key: &ClientResponseKey<WorldConnectResponse>,
    ) {
        self.world_connect_response_keys.remove(response_key);
    }

    pub fn world_connect_response_keys(
        &self,
    ) -> Vec<(ClientResponseKey<WorldConnectResponse>, UserKey)> {
        let mut out = Vec::new();
        for (res_key, usr_key) in self.world_connect_response_keys.iter() {
            out.push((res_key.clone(), *usr_key));
        }
        out
    }

    pub fn user_set_world_connected(
        &mut self,
        user_key: &UserKey,
        world_instance_secret: &str,
        user_id: UserId,
    ) {
        let user_data = self.users.get_mut(user_key).unwrap();
        user_data.set_world_connected(world_instance_secret);

        if !self.world_instances.contains_key(world_instance_secret) {
            self.world_instances
                .insert(world_instance_secret.to_string(), WorldInstanceData::new());
        }
        let world_instance = self.world_instances.get_mut(world_instance_secret).unwrap();
        world_instance.add_user(*user_key, user_id);
    }

    pub fn world_instance_exists(&self, world_instance_secret: &str) -> bool {
        self.world_instances.contains_key(world_instance_secret)
    }

    pub fn get_user_key_from_world_instance(
        &self,
        world_instance_secret: &str,
        user_id: &UserId,
    ) -> Option<UserKey> {
        let world_instance = self.world_instances.get(world_instance_secret)?;
        world_instance.user_id_to_key_map.get(user_id).copied()
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

    // Asset Server

    pub fn set_asset_server(&mut self, addr: &str, port: u16) {
        self.asset_server_opt = Some((addr.to_string(), port));
    }

    pub fn clear_asset_server(&mut self) {
        self.asset_server_opt = None;
    }

    pub fn get_asset_server_url(&self) -> Option<(String, u16)> {
        self.asset_server_opt
            .as_ref()
            .map(|(addr, port)| (addr.clone(), *port))
    }

    // Social Server

    pub fn set_social_server(&mut self, addr: &str, port: u16) {
        self.social_server_opt = Some((addr.to_string(), port));
    }

    pub fn clear_social_server(&mut self) {
        self.social_server_opt = None;
    }

    pub fn get_social_server_url(&self) -> Option<(String, u16)> {
        self.social_server_opt
            .as_ref()
            .map(|(addr, port)| (addr.clone(), *port))
    }
}
