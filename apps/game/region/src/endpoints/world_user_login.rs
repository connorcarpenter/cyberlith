
use log::{info, warn};
use http_client::HttpClient;

use http_server::{async_dup::Arc, Server, smol::lock::RwLock};

use region_server_http_proto::{
    WorldUserLoginRequest,
    WorldUserLoginResponse,
};
use world_server_http_proto::IncomingUserRequest;

use crate::state::State;

pub fn world_user_login(
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

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: WorldUserLoginRequest
) -> Result<WorldUserLoginResponse, ()> {
    info!("world user login request received from session server");

    let state = state.read().await;
    let world_server = state.get_available_world_server();
    let world_server_http_addr = world_server.http_addr();
    let world_server_signaling_addr = world_server.signal_addr();

    info!("sending incoming user request to world server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    let request = IncomingUserRequest::new(temp_region_secret, temp_token);

    let Ok(outgoing_response) = HttpClient::send(&world_server_http_addr, request).await else {
        warn!("Failed incoming user request to world server");
        return Err(());
    };

    info!("Received incoming user response from world server");

    info!("Sending user login response to session server");

    // TODO: end of part we need to get rid of

    Ok(WorldUserLoginResponse::new(world_server_signaling_addr, temp_token))
}
