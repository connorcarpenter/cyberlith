use log::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use config::REGION_SERVER_SECRET;
use region_server_http_proto::{
    WorldConnectRequest as RegionWorldConnectRequest,
    WorldConnectResponse as RegionWorldConnectResponse,
};
use world_server_http_proto::WorldConnectRequest as WorldWorldConnectRequest;

use crate::state::State;

pub fn world_connect(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: RegionWorldConnectRequest,
) -> Result<RegionWorldConnectResponse, ResponseError> {
    let state = state.read().await;

    info!(
        "incoming session server instance secret: {:?}",
        incoming_request.session_server_instance_secret
    );

    let Some(session_server) = state
        .get_session_server_from_instance_secret(&incoming_request.session_server_instance_secret)
    else {
        warn!("invalid session server instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    info!("world_connect request received from session server");

    let Some(world_server) = state.get_available_world_server() else {
        warn!("no available world server");
        return Err(ResponseError::InternalServerError(
            "no available world server".to_string(),
        ));
    };

    let session_server_addr = session_server.http_addr();
    let session_server_port = session_server.http_port();

    let world_server_instance_secret = world_server.instance_secret();
    let world_server_http_addr = world_server.http_addr();
    let world_server_http_port = world_server.http_port();
    let world_server_public_webrtc_url = world_server.public_webrtc_url();

    info!("sending world_connect request to world server");

    let temp_token = random::generate_random_string(16);

    let world_server_request = WorldWorldConnectRequest::new(
        REGION_SERVER_SECRET,
        session_server_addr,
        session_server_port,
        &temp_token,
    );

    let Ok(world_server_response) = HttpClient::send(
        world_server_http_addr,
        world_server_http_port,
        world_server_request,
    )
    .await
    else {
        warn!("failed incoming user request to world server");
        return Err(ResponseError::InternalServerError(
            "failed incoming user request to world server".to_string(),
        ));
    };

    info!("Received incoming user response from world server");

    let user_id = world_server_response.user_id;

    info!("Sending user login response to session server");

    Ok(RegionWorldConnectResponse::new(
        world_server_instance_secret,
        user_id,
        world_server_public_webrtc_url,
        &temp_token,
    ))
}
