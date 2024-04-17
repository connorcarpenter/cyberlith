use http_client::{HttpClient, ResponseError};
use http_server::{ApiServer, http_log_util, Server};

use auth_server_http_proto::UserRegisterRequest as AuthUserRegisterRequest;
use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, GATEWAY_SECRET};
use gateway_http_proto::{
    UserRegisterRequest as GatewayUserRegisterRequest,
    UserRegisterResponse as GatewayUserRegisterResponse,
};

pub fn user_register(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(
    incoming_request: GatewayUserRegisterRequest,
) -> Result<GatewayUserRegisterResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "user_register");

    http_log_util::send_req("gateway", "auth_server", "user_register");
    let auth_server_request = AuthUserRegisterRequest::new(
        GATEWAY_SECRET,
        &incoming_request.username,
        &incoming_request.email,
        &incoming_request.password,
    );
    let Ok(_auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "auth_server", "user_register");
    };
    http_log_util::recv_res("gateway", "auth_server", "user_register");

    http_log_util::send_res("gateway", "client", "user_register");
    Ok(GatewayUserRegisterResponse::new())
}
