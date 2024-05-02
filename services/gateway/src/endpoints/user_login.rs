use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use http_server::{ApiRequest, ApiResponse, Request, Response, ResponseError};
use http_client::HttpClient;

use auth_server_http_proto::UserLoginRequest;
use gateway_http_proto::{UserLoginRequest as GatewayUserLoginRequest, UserLoginResponse as GatewayUserLoginResponse, UserLoginResponse};

use crate::cookie_middleware::response_set_cookies_tokens;

pub(crate) async fn handler(
    _incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {
    let host_name = "gateway";
    let remote_name = "client";
    http_server::http_log_util::recv_req(host_name, remote_name, format!("{} {}", incoming_request.method.as_str(), &incoming_request.url).as_str());

    // parse out request
    let gateway_request = match GatewayUserLoginRequest::from_request(incoming_request) {
        Ok(r) => r,
        Err(e) => {
            http_server::http_log_util::send_res(host_name, e.to_string().as_str());
            return Err(ResponseError::SerdeError);
        }
    };

    // call auth server
    let auth_server = "auth_server";
    let auth_addr = AUTH_SERVER_RECV_ADDR;
    let auth_port = AUTH_SERVER_PORT;

    let auth_request = UserLoginRequest::new(&gateway_request.handle, &gateway_request.password);

    http_server::http_log_util::send_req(host_name, auth_server, UserLoginRequest::name());
    match HttpClient::send(&auth_addr, auth_port, auth_request).await {
        Ok(auth_response) => {
            http_server::http_log_util::recv_res(host_name, auth_server, UserLoginResponse::name());
            let access_token = auth_response.access_token;
            let refresh_token = auth_response.refresh_token;

            let mut gateway_response = GatewayUserLoginResponse::new().to_response();
            response_set_cookies_tokens(&mut gateway_response, &access_token, &refresh_token);

            http_server::http_log_util::send_res(host_name, GatewayUserLoginResponse::name());
            return Ok(gateway_response);
        },
        Err(e) => {
            match e {
                ResponseError::Unauthenticated => {
                    http_server::http_log_util::recv_res(host_name, auth_server, "unauthenticated");
                    http_server::http_log_util::send_res(host_name, "unauthenticated");
                    return Err(ResponseError::Unauthenticated);
                }
                e => {
                    http_server::http_log_util::recv_res(host_name, auth_server, "internal_server_error");
                    http_server::http_log_util::send_res(host_name, "internal_server_error");
                    return Err(ResponseError::InternalServerError(e.to_string()));
                }
            }
        }
    }
}