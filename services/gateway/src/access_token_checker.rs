use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use http_client::{HttpClient, ResponseError};
use http_server::{ApiRequest, ApiResponse, Request, Response};

use auth_server_http_proto::{AccessTokenValidateRequest, AccessTokenValidateResponse};
use logging::info;

pub(crate) async fn api_middleware(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Option<Result<Response, ResponseError>> {

    let access_token: Option<String> = incoming_request.get_header("authorization").map(|s| s.clone());
    if access_token.is_some() {
        info!("found access_token in header: {}", access_token.as_ref().unwrap());
    }
    middleware_impl(incoming_addr, incoming_request, access_token).await
}

pub(crate) async fn www_middleware(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Option<Result<Response, ResponseError>> {

    // get access token from cookie on incoming_request
    let access_token: Option<String> = if let Some(cookie) = incoming_request.get_header("cookie") {
        // parse 'access_token' out of cookie
        let token = cookie
            .split(';')
            .filter(|cookie| cookie.starts_with("access_token="))
            .map(|cookie| cookie.trim_start_matches("access_token=").to_string())
            .next();
        token
    } else {
        None
    };
    if access_token.is_some() {
        info!("found access_token in cookie: {}", access_token.as_ref().unwrap());
    }
    middleware_impl(incoming_addr, incoming_request, access_token).await
}

pub(crate) async fn api_base64_middleware(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Option<Result<Response, ResponseError>> {

    let access_token: Option<String> = get_access_token_from_base64(&incoming_request);
    if access_token.is_some() {
        info!("found access_token in header: {}", access_token.as_ref().unwrap());
    }
    middleware_impl(incoming_addr, incoming_request, access_token).await
}

fn get_access_token_from_base64(incoming_request: &Request) -> Option<String> {
    let auth_header = incoming_request.get_header("authorization").map(|s| s.clone())?;
    let auth_header = base64::decode(&auth_header).ok()?;
    let auth_header = String::from_utf8(auth_header).ok()?;
    Some(auth_header)
}

async fn middleware_impl(
    _incoming_addr: SocketAddr,
    incoming_request: Request,
    access_token_opt: Option<String>,
) -> Option<Result<Response, ResponseError>> {
    let host_name = "gateway_auth";
    let remote_name = "client";

    http_server::http_log_util::recv_req(host_name, remote_name, format!("{} {}", incoming_request.method.as_str(), &incoming_request.url).as_str());
    let Some(access_token) = access_token_opt else {
        http_server::http_log_util::send_res(host_name, "unauthenticated (no access_token found)");
        return Some(Err(ResponseError::Unauthenticated));
    };

    // call auth server to validate access token
    let auth_server = "auth_server";
    let remote_addr = AUTH_SERVER_RECV_ADDR;
    let remote_port = AUTH_SERVER_PORT;

    http_server::http_log_util::send_req(host_name, auth_server, AccessTokenValidateRequest::name());

    let validate_request = AccessTokenValidateRequest::new(&access_token);

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