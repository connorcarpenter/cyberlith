use logging::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, smol::lock::RwLock, ApiServer, Server};

use config::REGION_SERVER_SECRET;
use region_server_http_proto::{SessionConnectRequest, SessionConnectResponse};
use session_server_http_proto::IncomingUserRequest;

use crate::state::State;

pub fn session_connect(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    _incoming_request: SessionConnectRequest,
) -> Result<SessionConnectResponse, ResponseError> {

    info!("session_connect request received from gateway");

    let state = state.read().await;
    let Some(session_server) = state.get_available_session_server() else {
        warn!("no available session server");
        return Err(ResponseError::InternalServerError(
            "no available session server".to_string(),
        ));
    };
    let session_server_http_addr = session_server.http_addr();
    let session_server_http_port = session_server.http_port();
    let session_server_public_webrtc_url = session_server.public_webrtc_url();

    info!("sending session_connect request to session server");

    let temp_token = random::generate_random_string(16);

    let request = IncomingUserRequest::new(REGION_SERVER_SECRET, &temp_token);

    let Ok(_outgoing_response) =
        HttpClient::send(&session_server_http_addr, session_server_http_port, request).await
    else {
        warn!("Failed session_connect request to session server");
        return Err(ResponseError::InternalServerError(
            "failed session_connect to session server".to_string(),
        ));
    };

    info!("Received session_connect response from session server");

    info!("Sending session_connect response to gateway");

    Ok(SessionConnectResponse::new(
        &session_server_public_webrtc_url,
        &temp_token,
    ))
}
