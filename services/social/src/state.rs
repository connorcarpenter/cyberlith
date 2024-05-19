use std::{collections::HashSet, time::{Duration, Instant}};
use std::collections::hash_set::Iter;

pub enum ConnectionState {
    Disconnected,
    Connected,
}

pub struct State {
    region_server_connection_state: ConnectionState,
    region_server_last_sent: Instant,
    region_server_last_heard: Instant,
    registration_resend_rate: Duration,
    region_server_disconnect_timeout: Duration,

    session_servers: HashSet<(String, u16)>,
}

impl State {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            registration_resend_rate,
            region_server_disconnect_timeout,

            session_servers: HashSet::new(),
        }
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

    // Session Servers Server

    pub fn add_session_server(&mut self, addr: &str, port: u16) {
        self.session_servers.insert((addr.to_string(), port));
    }

    pub fn remove_session_server(&mut self, addr: &str, port: u16) {
        self.session_servers.remove(&(addr.to_string(), port));
    }

    pub fn session_servers(&self) -> Iter<'_, (String, u16)> {
        self.session_servers.iter()
    }
}
