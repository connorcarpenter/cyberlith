use log::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::Server;

use config::{GATEWAY_SECRET, AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use gateway_http_proto::{UserRegisterRequest as GatewayUserRegisterRequest, UserRegisterResponse as GatewayUserRegisterResponse};
use auth_server_http_proto::UserRegisterRequest as AuthUserRegisterRequest;

pub fn user_register(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(incoming_request: GatewayUserRegisterRequest) -> Result<GatewayUserRegisterResponse, ResponseError> {
    info!("user_register request <- client");

    info!("user_register request -> auth server");

    let auth_server_request = AuthUserRegisterRequest::new(
        GATEWAY_SECRET,
        &incoming_request.username,
        &incoming_request.email,
        &incoming_request.password,
    );
    let Ok(_auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        warn!("FAILED user_register request -> auth server!");
        return Err(ResponseError::InternalServerError(
            "failed user_register request to auth server".to_string(),
        ));
    };

    info!("user_register response <- auth server",);

    info!("user_register response -> client");

    Ok(GatewayUserRegisterResponse::new())
}
