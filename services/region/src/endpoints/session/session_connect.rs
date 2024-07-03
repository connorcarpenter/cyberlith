use config::REGION_SERVER_SECRET;
use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiRequest, ApiServer, Server};
use logging::warn;

use region_server_http_proto::{SessionConnectRequest, SessionConnectResponse};
use session_server_http_proto::IncomingUserRequest;
use social_server_http_proto::UserConnectedRequest;

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

    let user_id = incoming_request.user_id;

    let state = state.read().await;

    // send user connection request to social server
    {
        let Some(social_server) = state.get_social_server() else {
            warn!("no available social server");
            return Err(ResponseError::InternalServerError(
                "no available social server".to_string(),
            ));
        };
        let social_addr = social_server.http_addr();
        let social_port = social_server.http_port();
        let request_method = UserConnectedRequest::method();
        let request_path = UserConnectedRequest::path();

        let request = UserConnectedRequest::new(REGION_SERVER_SECRET, user_id);

        let host = "region";
        let remote = "social";
        let logged_remote_url = format!(
            "{} host:{}/{}",
            request_method.as_str(),
            social_port,
            request_path
        );
        http_server::log_util::send_req(&host, &remote, &logged_remote_url);
        let response_result = HttpClient::send(&social_addr, social_port, request).await;
        http_server::log_util::recv_res(&host, &remote, &logged_remote_url);

        let Ok(_response) = response_result else {
            warn!("Failed session_connect request to social server");
            return Err(ResponseError::InternalServerError(
                "failed session_connect to social server".to_string(),
            ));
        };
    }

    // send user connection request to session server
    {
        let Some(session_server) = state.get_available_session_server() else {
            warn!("no available session server");
            return Err(ResponseError::InternalServerError(
                "no available session server".to_string(),
            ));
        };
        let session_addr = session_server.http_addr();
        let session_port = session_server.http_port();
        let request_method = IncomingUserRequest::method();
        let request_path = IncomingUserRequest::path();

        let temp_token = random::generate_random_string(16);
        let request =
            IncomingUserRequest::new(REGION_SERVER_SECRET, user_id, &temp_token);

        let host = "region";
        let remote = "session";
        let logged_remote_url = format!(
            "{} host:{}/{}",
            request_method.as_str(),
            session_port,
            request_path
        );
        http_server::log_util::send_req(&host, &remote, &logged_remote_url);
        let response_result = HttpClient::send(&session_addr, session_port, request).await;
        http_server::log_util::recv_res(&host, &remote, &logged_remote_url);

        let Ok(_response) = response_result else {
            warn!("Failed session_connect request to session server");
            return Err(ResponseError::InternalServerError(
                "failed session_connect to session server".to_string(),
            ));
        };

        return Ok(SessionConnectResponse::new(&temp_token));
    }
}
