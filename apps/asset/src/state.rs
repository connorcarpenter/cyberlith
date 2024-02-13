
use std::time::{Duration, Instant};
use crate::asset_cache::AssetCache;

use crate::asset_map::AssetMap;

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
    asset_map: AssetMap,
    asset_cache: AssetCache,
}

impl State {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
        asset_cache_size_kb: u32,
        asset_map: AssetMap,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            registration_resend_rate,
            region_server_disconnect_timeout,
            asset_map,
            asset_cache: AssetCache::new(asset_cache_size_kb),
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

    pub fn asset_map(&self) -> &AssetMap {
        &self.asset_map
    }

    pub fn asset_cache_mut(&mut self) -> &mut AssetCache {
        &mut self.asset_cache
    }
}