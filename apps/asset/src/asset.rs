
use log::info;

use http_client::ResponseError;
use http_server::{Server, smol::lock::RwLock, async_dup::Arc};

use asset_server_http_proto::{AssetRequest, AssetResponse};

use crate::state::State;

pub fn endpoint(
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.endpoint(
        move |(_addr, req)| {
            let state = state.clone();
            async move {
                async_impl(state, req).await
            }
        }
    );
}

async fn async_impl(state: Arc<RwLock<State>>, request: AssetRequest) -> Result<AssetResponse, ResponseError> {
    info!("Asset request received: {:?}, sending response", request);
    let req_asset_id = request.asset_id();
    let req_etag_opt = request.etag_opt();
    let mut state = state.write().await;
    let asset_map = state.asset_map();
    if let Some(metadata) = asset_map.get(&req_asset_id) {
        let asset_etag = metadata.etag();
        if let Some(req_etag) = req_etag_opt {
            if asset_etag == req_etag {
                return Ok(AssetResponse::not_modified());
            }
        }

        let path = metadata.path().to_string();
        let Some(asset_data) = state.asset_cache_mut().load(&path) else {
            return Err(ResponseError::InternalServerError(format!("Failed to load asset data for path: `{}`", path)));
        };
        return Ok(AssetResponse::modified(asset_etag, asset_data));
    } else {
        return Err(ResponseError::NotFound);
    }
}
