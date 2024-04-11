use http_client::{HttpClient, ResponseError};
use http_server::{http_log_util, Server};

use auth_server_http_proto::UserPasswordResetRequest as AuthUserPasswordResetRequest;
use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, GATEWAY_SECRET};
use gateway_http_proto::{
    UserPasswordResetRequest as GatewayUserPasswordResetRequest,
    UserPasswordResetResponse as GatewayUserPasswordResetResponse,
};

pub fn user_password_reset(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(
    incoming_request: GatewayUserPasswordResetRequest,
) -> Result<GatewayUserPasswordResetResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "user_password_reset");

    http_log_util::send_req("gateway", "auth_server", "user_password_reset");
    let auth_server_request = AuthUserPasswordResetRequest::new(
        GATEWAY_SECRET,
        &incoming_request.reset_password_token,
        &incoming_request.new_password,
    );
    let Ok(_auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "auth_server", "user_password_reset");
    };
    http_log_util::recv_res("gateway", "auth_server", "user_password_reset");

    http_log_util::send_res("gateway", "client", "user_password_reset");
    Ok(GatewayUserPasswordResetResponse::new())
}
