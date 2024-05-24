use std::time::Duration;

use filecache::FileCache;
use http_client::ResponseError;

use asset_server_http_proto::{AssetRequest, AssetResponse};

use crate::{region_connection::RegionServerState, asset_metadata_store::AssetMetadataStore};

pub struct State {
    pub region_server: RegionServerState,
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
            region_server: RegionServerState::new(
                registration_resend_rate,
                region_server_disconnect_timeout,
            ),
            asset_metadata_store,
            asset_cache: FileCache::new(asset_cache_size_kb),
        }
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
