
use log::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::Server;

use config::{REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, ORCHESTRATOR_SECRET};
use orchestrator_http_proto::{LoginRequest, LoginResponse};
use region_server_http_proto::SessionUserLoginRequest;

pub fn world_user_login(
    server: &mut Server,
) {
    server.endpoint(
        move |(_addr, req)| {
            async move {
                async_impl(req).await
            }
        }
    );
}

async fn async_impl(incoming_request: LoginRequest) -> Result<LoginResponse, ResponseError> {
    info!("Login request received from client");

    info!("Sending login request to region server");

    let region_request = SessionUserLoginRequest::new(ORCHESTRATOR_SECRET, &incoming_request.username, &incoming_request.password);
    let Ok(region_response) = HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, region_request).await else {
        warn!("Failed login request to region server");
        return Err(ResponseError::InternalServerError("failed login request to region server".to_string()));
    };

    info!(
        "Received login response from region server: addr: {:?}, token: {}",
        region_response.session_server_addr,
        region_response.token,
    );

    info!("Sending login response to client");

    Ok(LoginResponse::new(
        &region_response.session_server_addr,
        region_response.session_server_port,
        &region_response.token,
    ))
}
