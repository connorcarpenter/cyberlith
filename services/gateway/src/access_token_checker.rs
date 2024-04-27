use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use http_client::{HttpClient, ResponseError};
use http_server::{ApiRequest, ApiResponse, Request, Response};

use auth_server_http_proto::{AccessTokenValidateRequest, AccessTokenValidateResponse};

pub(crate) async fn access_token_check(
    _incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Option<Result<Response, ResponseError>> {

    let host_name = "gateway auth";
    let remote_name = "client";

    http_server::http_log_util::recv_req(host_name, remote_name, format!("{} {}", incoming_request.method.as_str(), &incoming_request.url).as_str());
    let Some(access_token) = incoming_request.headers.get("authorization") else {
        http_server::http_log_util::send_res(host_name, "unauthenticated (no authorization header found)");
        return Some(Err(ResponseError::Unauthenticated));
    };

    // call auth server to validate access token
    let auth_server = "auth_server";
    let remote_addr = AUTH_SERVER_RECV_ADDR;
    let remote_port = AUTH_SERVER_PORT;

    http_server::http_log_util::send_req(host_name, auth_server, AccessTokenValidateRequest::name());

    let validate_request = AccessTokenValidateRequest::new(access_token);

    match HttpClient::send(&remote_addr, remote_port, validate_request).await {
        Ok(_validate_response) => {
            http_server::http_log_util::recv_res(host_name, auth_server, &AccessTokenValidateResponse::name());
            return None;
        },
        Err(e) => {
            match e {
                ResponseError::Unauthenticated => {
                    http_server::http_log_util::recv_res(host_name, auth_server, "unauthenticated");
                    http_server::http_log_util::send_res(host_name, "unauthenticated (invalid access token)");
                    return Some(Err(ResponseError::Unauthenticated));
                }
                e => {
                    http_server::http_log_util::recv_res(host_name, auth_server, "internal_server_error");
                    http_server::http_log_util::send_res(host_name, "internal_server_error");
                    return Some(Err(ResponseError::InternalServerError(e.to_string())));
                }
            }
        }
    }
}