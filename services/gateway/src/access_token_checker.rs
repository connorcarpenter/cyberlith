use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use http_client::{HttpClient, ResponseError};
use http_server::{ApiRequest, ApiResponse, Request, RequestMiddlewareAction, Response};

use auth_server_http_proto::{AccessToken, AccessTokenValidateRequest, AccessTokenValidateResponse};
use logging::info;

use crate::cookie_middleware::clear_cookie;

// pub(crate) async fn api_middleware(
//     incoming_addr: SocketAddr,
//     incoming_request: Request,
// ) -> Option<Result<Response, ResponseError>> {
//
//     let access_token: Option<String> = incoming_request.get_header("authorization").map(|s| s.clone());
//     if access_token.is_some() {
//         info!("found access_token in header: {}", access_token.as_ref().unwrap());
//     }
//     middleware_impl(incoming_addr, incoming_request, access_token).await
// }

pub(crate) async fn www_middleware(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {

    // get access token from cookie on incoming_request
    let access_token: Option<String> = if let Some(cookie) = incoming_request.get_header_first("cookie") {
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
    else {
        info!("no access_token found in cookie");
    }
    match middleware_impl(incoming_addr, incoming_request.clone(), access_token).await {
        RequestMiddlewareAction::Continue(incoming_request) => {
            // success
            RequestMiddlewareAction::Continue(incoming_request)
        },
        RequestMiddlewareAction::Error(e) => {
            let mut response = e.to_response(incoming_request.url.as_str());
            clear_cookie(&mut response);
            RequestMiddlewareAction::Stop(response)
        },
        RequestMiddlewareAction::Stop(mut response) => {
            clear_cookie(&mut response);
            RequestMiddlewareAction::Stop(response)
        }
    }
}

pub(crate) async fn www_middleware_redirect_home(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {
    www_middleware_redirect(incoming_addr, incoming_request, "/").await
}

async fn www_middleware_redirect(incoming_addr: SocketAddr, incoming_request: Request, new_url: &str) -> RequestMiddlewareAction {
    let url = incoming_request.url.clone();
    match www_middleware(incoming_addr, incoming_request).await {
        RequestMiddlewareAction::Continue(incoming_request) => {
            // success
            RequestMiddlewareAction::Continue(incoming_request)
        },
        RequestMiddlewareAction::Error(e) => match e {
            ResponseError::Unauthenticated => {
                // redirect to home page
                let mut response = Response::redirect(&url, new_url);
                clear_cookie(&mut response);
                RequestMiddlewareAction::Stop(response)
            },
            e => {
                // internal server error
                RequestMiddlewareAction::Error(e)
            },
        },
        RequestMiddlewareAction::Stop(outgoing_response) => {
            if outgoing_response.status == 401 {
                // redirect to home page
                let mut response = Response::redirect(&url, new_url);
                clear_cookie(&mut response);
                RequestMiddlewareAction::Stop(response)
            } else {
                // return previous response
                RequestMiddlewareAction::Stop(outgoing_response)
            }
        }
    }
}

pub(crate) async fn middleware_impl(
    _incoming_addr: SocketAddr,
    incoming_request: Request,
    access_token_opt: Option<String>,
) -> RequestMiddlewareAction {
    let host_name = "gateway_auth";
    let remote_name = "client";

    http_server::http_log_util::recv_req(host_name, remote_name, format!("{} {}", incoming_request.method.as_str(), &incoming_request.url).as_str());
    let Some(access_token) = access_token_opt else {
        http_server::http_log_util::send_res(host_name, "unauthenticated (no access token)");
        return RequestMiddlewareAction::Error(ResponseError::Unauthenticated);
    };
    if access_token.is_empty() {
        http_server::http_log_util::send_res(host_name, "unauthenticated (empty access token)");
        return RequestMiddlewareAction::Error(ResponseError::Unauthenticated);
    }

    // call auth server to validate access token
    let auth_server = "auth_server";
    let remote_addr = AUTH_SERVER_RECV_ADDR;
    let remote_port = AUTH_SERVER_PORT;

    let Some(access_token) = AccessToken::from_str(&access_token) else {
        return RequestMiddlewareAction::Error(ResponseError::Unauthenticated);
    };
    let validate_request = AccessTokenValidateRequest::new(access_token);

    http_server::http_log_util::send_req(host_name, auth_server, AccessTokenValidateRequest::name());
    match HttpClient::send(&remote_addr, remote_port, validate_request).await {
        Ok(_validate_response) => {
            http_server::http_log_util::recv_res(host_name, auth_server, &AccessTokenValidateResponse::name());
            return RequestMiddlewareAction::Continue(incoming_request);
        },
        Err(e) => {
            match e {
                ResponseError::Unauthenticated => {
                    http_server::http_log_util::recv_res(host_name, auth_server, "unauthenticated");
                    http_server::http_log_util::send_res(host_name, "unauthenticated (invalid access token)");
                    return RequestMiddlewareAction::Error(ResponseError::Unauthenticated);
                }
                e => {
                    http_server::http_log_util::recv_res(host_name, auth_server, "internal_server_error");
                    http_server::http_log_util::send_res(host_name, "internal_server_error");
                    return RequestMiddlewareAction::Error(ResponseError::InternalServerError(e.to_string()));
                }
            }
        }
    }
}