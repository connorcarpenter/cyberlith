use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, TargetEnv};
use http_client::{HttpClient};
use http_server::{ApiRequest, ApiResponse, extract_query_string, clear_query_string, Request, RequestMiddlewareAction, Response};

use auth_server_http_proto::{RegisterToken, UserRegisterConfirmRequest, UserRegisterConfirmResponse};
use crate::cookie_middleware::get_set_cookie_value;

pub(crate) async fn handle(
    _incoming_addr: SocketAddr,
    mut incoming_request: Request,
) -> RequestMiddlewareAction {
    let Some(query_string_args) = extract_query_string(&incoming_request.url) else {
        return RequestMiddlewareAction::Continue(incoming_request);
    };
    let Some(register_token) = query_string_args.get("register_token") else {
        return RequestMiddlewareAction::Continue(incoming_request);
    };
    if register_token.is_empty() {
        return RequestMiddlewareAction::Continue(incoming_request);
    }
    let Some(register_token) = RegisterToken::from_str(register_token) else {
        return RequestMiddlewareAction::Continue(incoming_request);
    };

    // call auth server to with register token
    let host_name = "gateway";
    let auth_server = "auth_server";
    let remote_addr = AUTH_SERVER_RECV_ADDR;
    let remote_port = AUTH_SERVER_PORT;

    http_server::http_log_util::send_req(host_name, auth_server, UserRegisterConfirmRequest::name());

    let confirm_request = UserRegisterConfirmRequest::new(register_token);

    match HttpClient::send(&remote_addr, remote_port, confirm_request).await {
        Ok(confirm_response) => {
            http_server::http_log_util::recv_res(host_name, auth_server, &UserRegisterConfirmResponse::name());

            let access_token = confirm_response.access_token;
            let refresh_token = confirm_response.refresh_token;

            let mut new_response = Response::redirect(&incoming_request.url, "/game");
            let now_utc = chrono::Utc::now();

            // put SetCookies in response

            // access token
            {
                let expire_time_utc_opt = match TargetEnv::get() {
                    TargetEnv::Local => None,
                    TargetEnv::Prod => {
                        let mut expire_time_utc = now_utc;
                        let expire_duration_1_week = chrono::Duration::days(1);
                        expire_time_utc += expire_duration_1_week;
                        Some(expire_time_utc)
                    }
                };
                let access_token_value = get_set_cookie_value("access_token", &access_token.to_string(), expire_time_utc_opt);
                new_response.insert_header(
                    "Set-Cookie",
                    &access_token_value,
                );
            }
            // refresh token
            {
                let expire_time_utc_opt = match TargetEnv::get() {
                    TargetEnv::Local => None,
                    TargetEnv::Prod => {
                        let mut expire_time_utc = now_utc;
                        let expire_duration_1_week = chrono::Duration::weeks(1);
                        expire_time_utc += expire_duration_1_week;
                        Some(expire_time_utc)
                    }
                };
                let refresh_token_value = get_set_cookie_value("refresh_token", &refresh_token.to_string(), expire_time_utc_opt);
                new_response.insert_header(
                    "Set-Cookie",
                    &refresh_token_value,
                );
            }

            return RequestMiddlewareAction::Stop(new_response);
        },
        Err(_e) => {
            clear_query_string(&mut incoming_request.url);
            return RequestMiddlewareAction::Continue(incoming_request);
        }
    }

    // send register token to auth server

}