use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use http_client::{HttpClient};
use http_server::{ApiRequest, ApiResponse, Request, RequestMiddlewareAction, Response};
use logging::info;

use auth_server_http_proto::{AccessToken, AccessTokenValidateRequest, AccessTokenValidateResponse, RefreshToken, RefreshTokenGrantRequest, RefreshTokenGrantResponse};

use crate::cookie_middleware::get_set_cookie_value;

pub(crate) async fn require_auth_tokens(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {

    // get access token from cookie on incoming_request
    match auth_impl(&incoming_addr, &incoming_request).await {
        AuthResult::Continue => {
            // success
            RequestMiddlewareAction::Continue(incoming_request, None)
        },
        AuthResult::ContinueAndNewAccessToken(access_token) => {
            const ONE_DAY_IN_SECONDS: u32 = 60 * 60 * 24;
            let access_token_value = get_set_cookie_value("access_token", &access_token.to_string(), ONE_DAY_IN_SECONDS);
            RequestMiddlewareAction::Continue(incoming_request, Some(access_token_value))
        },
        AuthResult::Stop(clear_access_token, clear_refresh_token) => {
            let mut response = Response::unauthenticated(&incoming_request.url);
            if clear_access_token {
                response.insert_header("Set-Cookie", &get_set_cookie_value("access_token", "", 0));
            }
            if clear_refresh_token {
                response.insert_header("Set-Cookie", &get_set_cookie_value("access_token", "", 0));
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
        AuthResult::Continue => {
            // success
            RequestMiddlewareAction::Continue(incoming_request, None)
        },
        AuthResult::ContinueAndNewAccessToken(access_token) => {
            const ONE_DAY_IN_SECONDS: u32 = 60 * 60 * 24;
            let access_token_value = get_set_cookie_value("access_token", &access_token.to_string(), ONE_DAY_IN_SECONDS);
            RequestMiddlewareAction::Continue(incoming_request, Some(access_token_value))
        },
        AuthResult::Stop(clear_access_token, clear_refresh_token) => {
            let mut response = Response::redirect(&url, new_url);
            if clear_access_token {
                response.insert_header("Set-Cookie", &get_set_cookie_value("access_token", "", 0));
            }
            if clear_refresh_token {
                response.insert_header("Set-Cookie", &get_set_cookie_value("access_token", "", 0));
            }
            RequestMiddlewareAction::Stop(response)
        }
    }
}

enum AuthResult {
    Continue,
    ContinueAndNewAccessToken(String),
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

                if let Ok(_validate_response) = validate_result {
                    return AuthResult::Continue;
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
                    return AuthResult::ContinueAndNewAccessToken(grant_response.access_token.to_string());
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
        // parse 'access_token' out of cookie
        let token = cookie
            .split(';')
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