
use log::{info, warn};
use http_client::HttpClient;

use http_server::{async_dup::Arc, Server, smol::lock::RwLock};

use region_server_http_proto::{
    SessionUserLoginRequest,
    SessionUserLoginResponse,
};

use session_server_http_proto::IncomingUserRequest;

use crate::state::State;

pub fn session_user_login(
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
    incoming_request: SessionUserLoginRequest
) -> Result<SessionUserLoginResponse, ()> {
    info!("session user login request received from orchestrator");

    let state = state.read().await;
    let session_server = state.get_available_session_server();
    let session_server_http_addr = session_server.http_addr();
    let session_server_signaling_addr = session_server.signal_addr();

    info!("Sending incoming user request to session server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    let request = IncomingUserRequest::new(temp_region_secret, temp_token);

    // TODO: this is the part we need to get rid of

    let Ok(outgoing_response) = HttpClient::send(&session_server_http_addr, request).await else {
        warn!("Failed incoming user request to session server");
        return Err(());
    };

    info!("Received incoming user response from session server");

    info!("Sending user login response to orchestrator");

    // TODO: end of part we need to get rid of

    Ok(SessionUserLoginResponse::new(session_server_signaling_addr, temp_token))
}