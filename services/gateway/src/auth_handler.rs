use std::net::SocketAddr;
use auth_server_types::UserId;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, PUBLIC_IP_ADDR, TargetEnv};
use http_client::{HttpClient};
use http_server::{ApiRequest, ApiResponse, Request, RequestMiddlewareAction, Response};
use logging::info;

use auth_server_http_proto::{AccessToken, AccessTokenValidateRequest, AccessTokenValidateResponse, RefreshToken, RefreshTokenGrantRequest, RefreshTokenGrantResponse};

pub(crate) async fn require_auth_tokens(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {

    // get access token from cookie on incoming_request
    match auth_impl(&incoming_addr, &incoming_request).await {
        AuthResult::Continue(user_id) => {
            // success
            let mut continuing_request = incoming_request;
            let user_id: u64 = user_id.into();
            continuing_request.insert_header("user_id", &user_id.to_string());

            RequestMiddlewareAction::Continue(continuing_request, None)
        },
        AuthResult::ContinueAndNewAccessToken(user_id, access_token) => {

            let mut cookies = Vec::new();

            let access_token_value = AccessToken::get_new_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod(), &access_token);

            cookies.push(access_token_value);

            let mut continuing_request = incoming_request;
            let user_id: u64 = user_id.into();
            continuing_request.insert_header("user_id", &user_id.to_string());

            RequestMiddlewareAction::Continue(continuing_request, Some(cookies))
        },
        AuthResult::Stop(clear_access_token, clear_refresh_token) => {
            let mut response = Response::unauthenticated(&incoming_request.url);
            if clear_access_token {
                let access_token_value = AccessToken::get_expire_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod());
                response.insert_header("Set-Cookie", &access_token_value);
            }
            if clear_refresh_token {
                let refresh_token_value = RefreshToken::get_expire_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod());
                response.insert_header("Set-Cookie", &refresh_token_value);
            }
            RequestMiddlewareAction::Stop(response)
        }
    }
}

pub(crate) async fn require_auth_tokens_or_redirect_home(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {
    require_auth_tokens_or_redirect(incoming_addr, incoming_request, "/").await
}

async fn require_auth_tokens_or_redirect(incoming_addr: SocketAddr, incoming_request: Request, new_url: &str) -> RequestMiddlewareAction {
    let url = incoming_request.url.clone();
    match auth_impl(&incoming_addr, &incoming_request).await {
        AuthResult::Continue(user_id) => {
            // success

            let mut continuing_request = incoming_request;
            let user_id: u64 = user_id.into();
            continuing_request.insert_header("user_id", &user_id.to_string());

            RequestMiddlewareAction::Continue(continuing_request, None)
        },
        AuthResult::ContinueAndNewAccessToken(user_id, access_token) => {

            let mut cookies_set = Vec::new();
            let access_token_value = AccessToken::get_new_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod(), &access_token);
            cookies_set.push(access_token_value);

            let mut continuing_request = incoming_request;
            let user_id: u64 = user_id.into();
            continuing_request.insert_header("user_id", &user_id.to_string());

            RequestMiddlewareAction::Continue(continuing_request, Some(cookies_set))
        },
        AuthResult::Stop(clear_access_token, clear_refresh_token) => {
            let mut response = Response::redirect(&url, new_url);
            if clear_access_token {
                let access_token_value = AccessToken::get_expire_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod());
                response.insert_header("Set-Cookie", &access_token_value);
            }
            if clear_refresh_token {
                let refresh_token_value = RefreshToken::get_expire_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod());
                response.insert_header("Set-Cookie", &refresh_token_value);
            }
            RequestMiddlewareAction::Stop(response)
        }
    }
}

pub(crate) async fn if_auth_tokens_redirect_game(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {
    if_auth_tokens_redirect(incoming_addr, incoming_request, "/game").await
}

async fn if_auth_tokens_redirect(incoming_addr: SocketAddr, incoming_request: Request, new_url: &str) -> RequestMiddlewareAction {
    let url = incoming_request.url.clone();
    match auth_impl(&incoming_addr, &incoming_request).await {
        AuthResult::Continue(_user_id) => {
            // success
            // info!("sending redirect to /game");
            let response = Response::redirect(&url, new_url);
            RequestMiddlewareAction::Stop(response)
        },
        AuthResult::ContinueAndNewAccessToken(_user_id, access_token) => {
            let mut response = Response::redirect(&url, new_url);

            let access_token_value = AccessToken::get_new_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod(), &access_token);

            response.insert_header("Set-Cookie", &access_token_value);

            // info!("sending redirect with cookie to /game");
            RequestMiddlewareAction::Stop(response)
        },
        AuthResult::Stop(clear_access_token, clear_refresh_token) => {
            let mut set_cookies = Vec::new();
            if clear_access_token {
                let access_token_value = AccessToken::get_expire_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod());
                set_cookies.push(access_token_value);
            }
            if clear_refresh_token {
                let refresh_token_value = RefreshToken::get_expire_cookie_value(PUBLIC_IP_ADDR, TargetEnv::is_prod());
                set_cookies.push(refresh_token_value);
            }
            // info!("continuing to /");
            RequestMiddlewareAction::Continue(incoming_request, Some(set_cookies))
        }
    }
}

enum AuthResult {
    Continue(UserId),
    ContinueAndNewAccessToken(UserId, String),
    // clear access token?, clear refresh token?
    Stop(bool, bool),
}

async fn auth_impl(
    _incoming_addr: &SocketAddr,
    incoming_request: &Request,
) -> AuthResult {
    let host_name = "gateway_auth";
    let remote_name = "client";
    let auth_server = "auth_server";
    let auth_addr = AUTH_SERVER_RECV_ADDR;
    let auth_port = AUTH_SERVER_PORT;

    let access_token_opt = extract_cookie_value(&incoming_request, "access_token");
    let refresh_token_opt = extract_cookie_value(&incoming_request, "refresh_token");

    http_server::http_log_util::recv_req(host_name, remote_name, format!("{} {}", incoming_request.method.as_str(), &incoming_request.url).as_str());

    if let Some(access_token) = &access_token_opt {
        if !access_token.is_empty() {
            if let Some(access_token) = AccessToken::from_str(&access_token) {
                let validate_request = AccessTokenValidateRequest::new(access_token);

                http_server::http_log_util::send_req(host_name, auth_server, AccessTokenValidateRequest::name());
                let validate_result = HttpClient::send(&auth_addr, auth_port, validate_request).await;
                http_server::http_log_util::recv_res(host_name, auth_server, AccessTokenValidateResponse::name());

                if let Ok(validate_response) = validate_result {
                    return AuthResult::Continue(validate_response.user_id);
                }
            }
        }
    }

    if let Some(refresh_token) = &refresh_token_opt {
        if !refresh_token.is_empty() {
            if let Some(refresh_token) = RefreshToken::from_str(&refresh_token) {
                let grant_request = RefreshTokenGrantRequest::new(refresh_token);

                http_server::http_log_util::send_req(host_name, auth_server, RefreshTokenGrantRequest::name());
                let grant_result = HttpClient::send(&auth_addr, auth_port, grant_request).await;
                http_server::http_log_util::recv_res(host_name, auth_server, RefreshTokenGrantResponse::name());

                if let Ok(grant_response) = grant_result {
                    return AuthResult::ContinueAndNewAccessToken(grant_response.user_id, grant_response.access_token.to_string());
                }
            }
        }
    }

    return AuthResult::Stop(access_token_opt.is_some(), refresh_token_opt.is_some());
}

fn extract_cookie_value(
    incoming_request: &Request,
    cookie_name: &str,
) -> Option<String> {
    let cookie_match = format!("{}=", cookie_name);
    let value: Option<String> = if let Some(cookie) = incoming_request.get_header_first("cookie") {
        // parse 'name' out of cookie
        let token = cookie
            .split("; ")
            .filter(|cookie| cookie.starts_with(&cookie_match))
            .map(|cookie| cookie.trim_start_matches(&cookie_match).to_string())
            .next();
        token
    } else {
        None
    };
    if value.is_some() {
        info!("found `{}` in cookie: `{}`", cookie_name, value.as_ref().unwrap());
    } else {
        info!("no `{}` found in cookie", cookie_name);
    }
    value
}