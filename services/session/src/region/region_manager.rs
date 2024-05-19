use std::time::{Duration, Instant};

use bevy_ecs::system::Resource;

use bevy_http_client::ResponseKey as ClientResponseKey;

use region_server_http_proto::SessionRegisterInstanceResponse;

pub enum ConnectionState {
    Disconnected,
    Connected,
}

#[derive(Resource)]
pub struct RegionManager {
    region_server_connection_state: ConnectionState,
    region_server_last_sent: Instant,
    region_server_last_heard: Instant,
    region_server_disconnect_timeout: Duration,
    registration_resend_rate: Duration,
    register_instance_response_key: Option<ClientResponseKey<SessionRegisterInstanceResponse>>,
}

impl RegionManager {
    pub fn new(
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
        }
    }

    // Region Server stuff
    pub fn register_instance_response_key(
        &self,
    ) -> Option<&ClientResponseKey<SessionRegisterInstanceResponse>> {
        self.register_instance_response_key.as_ref()
    }

    pub fn set_register_instance_response_key(
        &mut self,
        response_key: ClientResponseKey<SessionRegisterInstanceResponse>,
    ) {
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
        // self.clear_asset_server();
    }
}
