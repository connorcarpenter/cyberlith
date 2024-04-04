
use http_client::{HttpClient, ResponseError};
use http_server::{http_log_util, Server};

use config::{GATEWAY_SECRET, AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use gateway_http_proto::{UserLoginRequest as GatewayUserLoginRequest, UserLoginResponse as GatewayUserLoginResponse};
use auth_server_http_proto::UserLoginRequest as AuthUserLoginRequest;

pub fn user_login(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(incoming_request: GatewayUserLoginRequest) -> Result<GatewayUserLoginResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "user_login");

    http_log_util::send_req("gateway", "auth_server", "user_login");
    let auth_server_request = AuthUserLoginRequest::new(
        GATEWAY_SECRET,
        &incoming_request.handle,
        &incoming_request.password,
    );
    let Ok(_auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "auth_server", "user_login");
    };
    http_log_util::recv_res("gateway", "auth_server", "user_login");

    http_log_util::send_res("gateway", "client", "user_login");
    Ok(GatewayUserLoginResponse::new("faketoken"))
}
