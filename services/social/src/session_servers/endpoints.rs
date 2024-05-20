use auth_server_types::UserId;
use config::REGION_SERVER_SECRET;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::{info, warn};

use social_server_http_proto::{
    ConnectSessionServerRequest, ConnectSessionServerResponse, DisconnectSessionServerRequest,
    DisconnectSessionServerResponse,
};

use crate::state::State;

pub fn recv_connect_session_server_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_connect_session_server_request_impl(state, req).await }
    });
}

async fn async_recv_connect_session_server_request_impl(
    state: Arc<RwLock<State>>,
    request: ConnectSessionServerRequest,
) -> Result<ConnectSessionServerResponse, ResponseError> {
    if request.region_secret() != REGION_SERVER_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    info!("Connect Session Server request received from region server");

    let mut state = state.write().await;

    // setting last heard
    state.region_server.heard_from_region_server();

    // get full chat log
    let messages: Vec<(UserId, String)> = state.global_chat.get_full_log().iter().map(|m| m.clone()).collect();

    // store session server details
    state
        .session_servers
        .init_instance(
            request.session_secret(),
            request.http_addr(),
            request.http_port(),
            messages,
        ).await;

    // responding
    // info!("Sending connect social server response to region server ..");
    return Ok(ConnectSessionServerResponse);
}

pub fn recv_disconnect_session_server_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_disconnect_session_server_request_impl(state, req).await }
    });
}

async fn async_recv_disconnect_session_server_request_impl(
    state: Arc<RwLock<State>>,
    request: DisconnectSessionServerRequest,
) -> Result<DisconnectSessionServerResponse, ResponseError> {
    if request.region_secret() != REGION_SERVER_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    info!("Disconnect Session Server request received from region server");

    let mut state = state.write().await;

    // setting last heard
    state.region_server.heard_from_region_server();

    // erase session server details
    state
        .session_servers
        .remove_instance(request.session_secret());

    // responding
    // info!("Sending connect session server response to region server ..");
    return Ok(DisconnectSessionServerResponse);
}
