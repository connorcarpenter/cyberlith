use std::{time::{Instant, Duration}, collections::HashSet};

use bevy_ecs::system::Resource;

use naia_bevy_server::RoomKey;

use bevy_http_client::{ResponseKey as ClientResponseKey};

use region_server_http_proto::WorldRegisterInstanceResponse;

pub enum ConnectionState {
    Disconnected,
    Connected,
}

#[derive(Resource)]
pub struct Global {
    region_server_connection_state: ConnectionState,
    region_server_last_sent: Instant,
    region_server_last_heard: Instant,
    register_instance_response_key: Option<ClientResponseKey<WorldRegisterInstanceResponse>>,
    registration_resend_rate: Duration,
    region_server_disconnect_timeout: Duration,
    login_tokens: HashSet<String>,
    main_room_key: RoomKey,
}

impl Global {

    pub fn new(
        main_room_key: RoomKey,
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            register_instance_response_key: None,
            registration_resend_rate,
            region_server_disconnect_timeout,
            login_tokens: HashSet::new(),
            main_room_key
        }
    }

    pub fn register_instance_response_key(&self) -> Option<&ClientResponseKey<WorldRegisterInstanceResponse>> {
        self.register_instance_response_key.as_ref()
    }

    pub fn set_register_instance_response_key(&mut self, response_key: ClientResponseKey<WorldRegisterInstanceResponse>) {
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

    // Client login

    pub fn add_login_token(&mut self, token: &str) {
        self.login_tokens.insert(token.to_string());
    }

    pub fn take_login_token(&mut self, token: &str) -> bool {
        self.login_tokens.remove(token)
    }

    //

    pub fn main_room_key(&self) -> RoomKey {
        self.main_room_key
    }
}