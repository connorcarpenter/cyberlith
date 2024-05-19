use std::net::SocketAddr;

use config::{TargetEnv, AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, PUBLIC_IP_ADDR};
use http_client::HttpClient;
use http_server::{
    clear_query_string, extract_query_string, ApiRequest, ApiResponse, Request,
    RequestMiddlewareAction, Response,
};

use auth_server_http_proto::{
    AccessToken, RefreshToken, RegisterToken, UserRegisterConfirmRequest,
    UserRegisterConfirmResponse,
};

pub(crate) async fn handle(
    _incoming_addr: SocketAddr,
    mut incoming_request: Request,
) -> RequestMiddlewareAction {
    let Some(query_string_args) = extract_query_string(&incoming_request.url) else {
        return RequestMiddlewareAction::Continue(incoming_request, None);
    };
    let Some(register_token) = query_string_args.get("register_token") else {
        return RequestMiddlewareAction::Continue(incoming_request, None);
    };
    if register_token.is_empty() {
        return RequestMiddlewareAction::Continue(incoming_request, None);
    }
    let Some(register_token) = RegisterToken::from_str(register_token) else {
        return RequestMiddlewareAction::Continue(incoming_request, None);
    };

    // call auth server to with register token
    let host_name = "gateway";
    let auth_server = "auth_server";
    let remote_addr = AUTH_SERVER_RECV_ADDR;
    let remote_port = AUTH_SERVER_PORT;

    http_server::http_log_util::send_req(
        host_name,
        auth_server,
        UserRegisterConfirmRequest::name(),
    );

    let confirm_request = UserRegisterConfirmRequest::new(register_token);

    match HttpClient::send(&remote_addr, remote_port, confirm_request).await {
        Ok(confirm_response) => {
            http_server::http_log_util::recv_res(
                host_name,
                auth_server,
                &UserRegisterConfirmResponse::name(),
            );

            let access_token = confirm_response.access_token;
            let refresh_token = confirm_response.refresh_token;

            let mut new_response = Response::redirect(&incoming_request.url, "/game");

            // set access token
            let access_token_value = AccessToken::get_new_cookie_value(
                PUBLIC_IP_ADDR,
                TargetEnv::is_prod(),
                &access_token.to_string(),
            );
            new_response.insert_header("Set-Cookie", &access_token_value);

            // set refresh token
            let refresh_token_value = RefreshToken::get_new_cookie_value(
                PUBLIC_IP_ADDR,
                TargetEnv::is_prod(),
                &refresh_token.to_string(),
            );
            new_response.insert_header("Set-Cookie", &refresh_token_value);

            return RequestMiddlewareAction::Stop(new_response);
        }
        Err(_e) => {
            clear_query_string(&mut incoming_request.url);
            return RequestMiddlewareAction::Continue(incoming_request, None);
        }
    }

    // send register token to auth server
}
