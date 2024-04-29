use std::net::SocketAddr;

use auth_server_http_proto::{UserLoginRequest, UserLoginResponse};

use config::{
    AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR,
};
use http_client::ResponseError;
use http_server::{ApiRequest, ApiResponse, Request, Response};
use logging::warn;

use crate::target_env::{get_env, TargetEnv};

pub(crate) async fn handler(
    _incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {

    let host_name = "gateway";
    let auth_server = "auth_server";
    let remote_addr = AUTH_SERVER_RECV_ADDR;
    let remote_port = AUTH_SERVER_PORT;
    let remote_path = UserLoginRequest::path();

    http_server::http_log_util::send_req(host_name, auth_server, UserLoginRequest::name());

    let mut auth_request = incoming_request.clone();
    auth_request.url = format!("http://{}:{}/{}", remote_addr, remote_port, remote_path);
    match http_client::raw::fetch_async(auth_request).await {
        Ok(auth_response) => {
            http_server::http_log_util::recv_res(host_name, auth_server, UserLoginResponse::name());

            let mut outgoing_response = auth_response.clone();

            // read response, parse to UserLoginResponse
            let Ok(auth_response) = UserLoginResponse::from_response(auth_response) else {
                return Err(ResponseError::SerdeError);
            };

            outgoing_response.url = incoming_request.url;

            // put access token into user cookie

            let cookie_attributes = match get_env() {
                TargetEnv::Local => "".to_string(),
                TargetEnv::Prod => {
                    let mut expire_time_utc = chrono::Utc::now();
                    let expire_duration_1_week = chrono::Duration::weeks(1);
                    expire_time_utc += expire_duration_1_week;

                    format!(
                        "; Secure; HttpOnly; SameSite=Lax; Domain=.cyberlith.com; Expires={}",
                        expire_time_utc
                    )
                },
            };

            let set_cookie_value = format!(
                "access_token={}{}",
                auth_response.access_token,
                cookie_attributes,
            );
            outgoing_response.set_header(
                "Set-Cookie",
                &set_cookie_value,
            );

            // info!("responding with Set-Cookie of value: {}", set_cookie_value);

            return Ok(outgoing_response);
        }
        Err(err) => {
            warn!(
                "Failed user login request to auth server: {}",
                err.to_string()
            );
            return Err(ResponseError::InternalServerError(
                "internal server error".to_string(),
            ));
        }
    }
}