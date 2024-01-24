
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
    _state: Arc<RwLock<State>>,
) {
    server.endpoint(
        move |(_addr, req)| {
            async move {
                async_impl(req).await
            }
        }
    );
}

async fn async_impl(incoming_request: WorldUserLoginRequest) -> Result<WorldUserLoginResponse, ()> {
    info!("world user login request received from session server");

    info!("sending incoming user request to world server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    // TODO: this is the part we need to get rid of

    let request = IncomingUserRequest::new(temp_region_secret, temp_token);
    let world_server_http_addr = "127.0.0.1:14202".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&world_server_http_addr, request).await else {
        warn!("Failed incoming user request to world server");
        return Err(());
    };

    info!("Received incoming user response from world server");

    info!("Sending user login response to session server");

    let world_server_signaling_addr = "127.0.0.1:14203".parse().unwrap();

    // TODO: end of part we need to get rid of

    Ok(WorldUserLoginResponse::new(world_server_signaling_addr, temp_token))
}
