use std::time::{Duration, Instant};

use filecache::FileCache;
use http_client::ResponseError;

use asset_server_http_proto::{AssetRequest, AssetResponse};

use crate::asset_metadata_store::AssetMetadataStore;

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
    asset_metadata_store: AssetMetadataStore,
    asset_cache: FileCache,
}

impl State {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
        asset_cache_size_kb: u32,
        asset_metadata_store: AssetMetadataStore,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            registration_resend_rate,
            region_server_disconnect_timeout,
            asset_metadata_store,
            asset_cache: FileCache::new(asset_cache_size_kb),
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

    pub fn handle_asset_request(
        &mut self,
        request: AssetRequest,
    ) -> Result<AssetResponse, ResponseError> {
        let req_asset_id = request.asset_id();
        let req_etag_opt = request.etag_opt();

        if let Some(metadata) = self.asset_metadata_store.get(&req_asset_id) {
            let asset_etag = metadata.etag();
            if let Some(req_etag) = req_etag_opt {
                if asset_etag == req_etag {
                    return Ok(AssetResponse::not_modified());
                }
            }

            let path = metadata.path().to_string();
            let Some(asset_data) = self.asset_cache.load(&path) else {
                return Err(ResponseError::InternalServerError(format!(
                    "Failed to load asset data for path: `{}`",
                    path
                )));
            };

            let asset_type = metadata.asset_type();

            let dependencies = metadata.dependencies().clone();
            return Ok(AssetResponse::modified(
                asset_etag,
                asset_type,
                dependencies,
                asset_data,
            ));
        } else {
            return Err(ResponseError::NotFound);
        }
    }
}
