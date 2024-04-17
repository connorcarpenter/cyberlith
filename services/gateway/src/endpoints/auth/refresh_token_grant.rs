use http_client::{HttpClient, ResponseError};
use http_server::{ApiServer, http_log_util, Server};

use auth_server_http_proto::RefreshTokenGrantRequest as AuthRefreshTokenGrantRequest;
use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, GATEWAY_SECRET};
use gateway_http_proto::{
    RefreshTokenGrantRequest as GatewayRefreshTokenGrantRequest,
    RefreshTokenGrantResponse as GatewayRefreshTokenGrantResponse,
};

pub fn refresh_token_grant(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(
    incoming_request: GatewayRefreshTokenGrantRequest,
) -> Result<GatewayRefreshTokenGrantResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "refresh_token_grant");

    http_log_util::send_req("gateway", "auth_server", "refresh_token_grant");
    let auth_server_request =
        AuthRefreshTokenGrantRequest::new(GATEWAY_SECRET, &incoming_request.refresh_token);
    let Ok(auth_server_response) =
        HttpClient::send(AUTH_SERVER_RECV_ADDR, AUTH_SERVER_PORT, auth_server_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "auth_server", "refresh_token_grant");
    };
    http_log_util::recv_res("gateway", "auth_server", "refresh_token_grant");

    http_log_util::send_res("gateway", "client", "refresh_token_grant");
    Ok(GatewayRefreshTokenGrantResponse::new(
        auth_server_response.access_token.as_str(),
    ))
}
