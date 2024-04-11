use http_client::{HttpClient, ResponseError};
use http_server::{http_log_util, Server};

use auth_server_http_proto::UserPasswordForgotRequest as AuthUserPasswordForgotRequest;
use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, GATEWAY_SECRET};
use gateway_http_proto::{
    UserPasswordForgotRequest as GatewayUserPasswordForgotRequest,
    UserPasswordForgotResponse as GatewayUserPasswordForgotResponse,
};

pub fn user_password_forgot(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(
    incoming_request: GatewayUserPasswordForgotRequest,
) -> Result<GatewayUserPasswordForgotResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "user_password_forgot");

    http_log_util::send_req("gateway", "auth_server", "user_password_forgot");
    let auth_server_request =
        AuthUserPasswordForgotRequest::new(GATEWAY_SECRET, &incoming_request.email);
    let Ok(_auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "auth_server", "user_password_forgot");
    };
    http_log_util::recv_res("gateway", "auth_server", "user_password_forgot");

    http_log_util::send_res("gateway", "client", "user_password_forgot");
    Ok(GatewayUserPasswordForgotResponse::new())
}
