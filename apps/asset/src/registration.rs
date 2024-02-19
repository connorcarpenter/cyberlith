use log::{info, warn};

use http_client::HttpClient;
use http_server::{async_dup::Arc, smol::lock::RwLock};

use region_server_http_proto::AssetRegisterInstanceRequest;

use config::{ASSET_SERVER_PORT, ASSET_SERVER_RECV_ADDR, ASSET_SERVER_GLOBAL_SECRET, REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT};

use crate::state::State;

pub async fn handle(state: Arc<RwLock<State>>) {
    let mut state = state.write().await;

    if state.connected() {
        return;
    }
    if !state.time_to_resend_registration() {
        return;
    }

    let request = AssetRegisterInstanceRequest::new(
        ASSET_SERVER_GLOBAL_SECRET,
        ASSET_SERVER_RECV_ADDR,
        ASSET_SERVER_PORT,
    );
    let response = HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request).await;
    match response {
        Ok(_) => {
            info!("from {:?}:{} - asset server registration success", REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT);
            state.set_connected();
        },
        Err(err) => {
            warn!("from {:?}:{} - asset server registration failure: {}", REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, err.to_string());
        }
    }

    state.sent_to_region_server();
}