use std::{collections::HashMap, time::{Instant, Duration}};
use std::collections::HashSet;

use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use bevy_http_client::{ResponseKey as ClientResponseKey};

use region_server_http_proto::{SessionRegisterInstanceResponse, WorldUserLoginResponse};

pub enum ConnectionState {
    Disconnected,
    Connected,
}

#[derive(Resource)]
pub struct Global {
    region_server_connection_state: ConnectionState,
    region_server_last_sent: Instant,
    region_server_last_heard: Instant,
    register_instance_response_key: Option<ClientResponseKey<SessionRegisterInstanceResponse>>,
    world_connect_response_keys: HashMap<ClientResponseKey<WorldUserLoginResponse>, UserKey>,
    registration_resend_rate: Duration,
    region_server_disconnect_timeout: Duration,
    world_connect_resend_rate: Duration,
    login_tokens: HashSet<String>,
    worldless_users: HashMap<UserKey, Option<Instant>>,
}

impl Global {

    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
        world_connect_resend_rate: Duration,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            register_instance_response_key: None,
            world_connect_response_keys: HashMap::new(),
            registration_resend_rate,
            region_server_disconnect_timeout,
            world_connect_resend_rate,
            login_tokens: HashSet::new(),
            worldless_users: HashMap::new(),
        }
    }

    // Region Server stuff
    pub fn register_instance_response_key(&self) -> Option<&ClientResponseKey<SessionRegisterInstanceResponse>> {
        self.register_instance_response_key.as_ref()
    }

    pub fn set_register_instance_response_key(&mut self, response_key: ClientResponseKey<SessionRegisterInstanceResponse>) {
        self.register_instance_response_key = Some(response_key);
    }

    pub fn clear_register_instance_response_key(&mut self) {
        self.register_instance_response_key = None;
    }

    pub fn waiting_for_registration_response(&self) -> bool {
        self.register_instance_response_key.is_some()
    }

    pub fn time_to_resend_registration(&self) -> bool {
        let time_since_last_sent = self.region_server_last_sent.elapsed();
        time_since_last_sent >= self.registration_resend_rate
    }

    pub fn time_to_disconnect(&self) -> bool {
        let time_since_last_heard = self.region_server_last_heard.elapsed();
        time_since_last_heard >= self.region_server_disconnect_timeout
    }

    pub fn heard_from_region_server(&mut self) {
        self.region_server_last_heard = Instant::now();
    }

    pub fn sent_to_region_server(&mut self) {
        self.region_server_last_sent = Instant::now();
    }

    pub fn connected(&self) -> bool {
        match self.region_server_connection_state {
            ConnectionState::Connected => true,
            ConnectionState::Disconnected => false,
        }
    }

    pub fn set_connected(&mut self) {
        self.region_server_connection_state = ConnectionState::Connected;
        self.heard_from_region_server();
    }

    pub fn set_disconnected(&mut self) {
        self.region_server_connection_state = ConnectionState::Disconnected;
    }

    // World Keys

    pub fn init_worldless_user(&mut self, user_key: &UserKey) {
        self.worldless_users.insert(*user_key, None);
    }

    pub fn add_worldless_user(&mut self, user_key: &UserKey) {
        self.worldless_users.insert(*user_key, Some(Instant::now()));
    }

    pub fn take_worldless_users(&mut self) -> Vec<UserKey> {
        let now = Instant::now();

        let mut worldless_users = Vec::new();
        for (user_key, last_sent_opt) in self.worldless_users.iter() {
            if let Some(last_sent) = last_sent_opt {
                let time_since_last_sent = now.duration_since(*last_sent);
                if time_since_last_sent >= self.world_connect_resend_rate {
                    worldless_users.push(*user_key);
                }
            } else {
                worldless_users.push(*user_key);
            }
        }
        for user_key in worldless_users.iter() {
            self.worldless_users.remove(user_key);
        }
        worldless_users
    }

    pub fn add_world_key(&mut self, user_key: &UserKey, response_key: ClientResponseKey<WorldUserLoginResponse>) {
        self.world_connect_response_keys.insert(response_key, user_key.clone());
    }

    pub fn remove_world_key(&mut self, response_key: &ClientResponseKey<WorldUserLoginResponse>) {
        self.world_connect_response_keys.remove(response_key);
    }

    pub fn world_keys(&self) -> impl Iterator<Item = (&ClientResponseKey<WorldUserLoginResponse>, &UserKey)> {
        self.world_connect_response_keys.iter()
    }

    // Client login

    pub fn add_login_token(&mut self, token: &str) {
        self.login_tokens.insert(token.to_string());
    }

    pub fn take_login_token(&mut self, token: &str) -> bool {
        self.login_tokens.remove(token)
    }
}