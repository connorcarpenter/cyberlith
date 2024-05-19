use std::{collections::{HashSet, hash_set::Iter}, time::Duration};

use crate::region_server_state::RegionServerState;

pub struct State {
    pub region_server: RegionServerState,
    session_servers: HashSet<(String, u16)>,
}

impl State {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            region_server: RegionServerState::new(registration_resend_rate, region_server_disconnect_timeout),
            session_servers: HashSet::new(),
        }
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
