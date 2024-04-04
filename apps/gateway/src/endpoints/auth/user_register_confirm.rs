
use http_client::{HttpClient, ResponseError};
use http_server::{http_log_util, Server};

use config::{GATEWAY_SECRET, AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use gateway_http_proto::{UserRegisterConfirmRequest as GatewayUserRegisterConfirmRequest, UserRegisterConfirmResponse as GatewayUserRegisterConfirmResponse};
use auth_server_http_proto::UserRegisterConfirmRequest as AuthUserRegisterConfirmRequest;

pub fn user_register_confirm(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(incoming_request: GatewayUserRegisterConfirmRequest) -> Result<GatewayUserRegisterConfirmResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "user_register_confirm");

    http_log_util::send_req("gateway", "auth_server", "user_register_confirm");
    let auth_server_request = AuthUserRegisterConfirmRequest::new(
        GATEWAY_SECRET,
        &incoming_request.register_token,
    );
    let Ok(_auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "auth_server", "user_register_confirm");
    };
    http_log_util::recv_res("gateway", "auth_server", "user_register_confirm");

    http_log_util::send_res("gateway", "client", "user_register_confirm");
    Ok(GatewayUserRegisterConfirmResponse::new("faketoken"))
}
