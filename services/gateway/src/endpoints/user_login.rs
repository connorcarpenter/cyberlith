use std::net::SocketAddr;

use config::{TargetEnv, AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, PUBLIC_IP_ADDR};
use http_client::HttpClient;
use http_server::{ApiRequest, ApiResponse, Request, Response, ResponseError};

use auth_server_http_proto::{
    AccessToken, RefreshToken, UserLoginRequest as AuthUserLoginRequest,
    UserLoginResponse as AuthUserLoginResponse,
};
use gateway_http_proto::{
    UserLoginRequest as GatewayUserLoginRequest, UserLoginResponse as GatewayUserLoginResponse,
};

use crate::demultiply_handler::{get_user_online_status_impl, UserPresenceResult};

pub(crate) async fn handler(
    _incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {
    let host_name = "gateway";
    let remote_name = "client";
    http_server::log_util::recv_req(
        host_name,
        remote_name,
        format!(
            "{} {}",
            incoming_request.method.as_str(),
            &incoming_request.url
        )
        .as_str(),
    );

    // parse out request
    let gateway_request = match GatewayUserLoginRequest::from_request(incoming_request) {
        Ok(r) => r,
        Err(e) => {
            http_server::log_util::send_res(host_name, e.to_string().as_str());
            return Err(ResponseError::SerdeError);
        }
    };

    // call auth server
    let auth_server = "auth_server";
    let auth_addr = AUTH_SERVER_RECV_ADDR;
    let auth_port = AUTH_SERVER_PORT;

    let auth_request =
        AuthUserLoginRequest::new(&gateway_request.handle, &gateway_request.password);

    http_server::log_util::send_req(host_name, auth_server, AuthUserLoginRequest::name());
    let (access_token, refresh_token, user_id) =
        match HttpClient::send(&auth_addr, auth_port, auth_request).await {
            Ok(auth_response) => {
                http_server::log_util::recv_res(
                    host_name,
                    auth_server,
                    AuthUserLoginResponse::name(),
                );
                let access_token = auth_response.access_token;
                let refresh_token = auth_response.refresh_token;
                let user_id = auth_response.user_id;

                (access_token, refresh_token, user_id)
            }
            Err(e) => match e {
                ResponseError::Unauthenticated => {
                    http_server::log_util::recv_res(host_name, auth_server, "unauthenticated");
                    http_server::log_util::send_res(host_name, "unauthenticated");
                    return Err(ResponseError::Unauthenticated);
                }
                e => {
                    http_server::log_util::recv_res(
                        host_name,
                        auth_server,
                        "internal_server_error",
                    );
                    http_server::log_util::send_res(host_name, "internal_server_error");
                    return Err(ResponseError::InternalServerError(e.to_string()));
                }
            },
        };

    let gateway_response = {
        // check for simultaneous login
        let presence_result = get_user_online_status_impl(user_id).await;
        match presence_result {
            UserPresenceResult::UserIsOnline => {
                GatewayUserLoginResponse::simultaneous_login_detected()
            }
            UserPresenceResult::UserIsOffline => GatewayUserLoginResponse::success(),
            UserPresenceResult::ServerError => {
                http_server::log_util::send_res(host_name, "internal_server_error");
                return Err(ResponseError::InternalServerError(
                    "social server error".to_string(),
                ));
            }
            _ => {
                panic!("should be impossible");
            }
        }
    };

    let mut gateway_response = gateway_response.to_response();
    let access_token_value = AccessToken::get_new_cookie_value(
        PUBLIC_IP_ADDR,
        TargetEnv::is_prod(),
        &access_token.to_string(),
    );
    gateway_response.insert_header("Set-Cookie", &access_token_value);
    let refresh_token_value = RefreshToken::get_new_cookie_value(
        PUBLIC_IP_ADDR,
        TargetEnv::is_prod(),
        &refresh_token.to_string(),
    );
    gateway_response.insert_header("Set-Cookie", &refresh_token_value);

    http_server::log_util::send_res(host_name, GatewayUserLoginResponse::name());
    return Ok(gateway_response);
}
