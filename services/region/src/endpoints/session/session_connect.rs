use config::REGION_SERVER_SECRET;
use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiRequest, ApiServer, Server};
use logging::warn;

use region_server_http_proto::{SessionConnectRequest, SessionConnectResponse};
use session_server_http_proto::IncomingUserRequest;

use crate::state::State;

pub fn session_connect(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: SessionConnectRequest,
) -> Result<SessionConnectResponse, ResponseError> {

    let state = state.read().await;
    let Some(session_server) = state.get_available_session_server() else {
        warn!("no available session server");
        return Err(ResponseError::InternalServerError(
            "no available session server".to_string(),
        ));
    };
    let remote_addr = session_server.http_addr();
    let remote_port = session_server.http_port();
    let remote_method = IncomingUserRequest::method();
    let remote_path = IncomingUserRequest::path();

    let temp_token = random::generate_random_string(16);
    let request =
        IncomingUserRequest::new(REGION_SERVER_SECRET, incoming_request.user_id, &temp_token);

    let host = "region";
    let remote = "session";
    let logged_remote_url = format!(
        "{} host:{}/{}",
        remote_method.as_str(),
        remote_port,
        remote_path
    );
    http_server::log_util::send_req(&host, &remote, &logged_remote_url);
    let result = HttpClient::send(&remote_addr, remote_port, request).await;
    http_server::log_util::recv_res(&host, &remote, &logged_remote_url);

    let Ok(_outgoing_response) = result else {
        warn!("Failed session_connect request to session server");
        return Err(ResponseError::InternalServerError(
            "failed session_connect to session server".to_string(),
        ));
    };

    Ok(SessionConnectResponse::new(&temp_token))
}
