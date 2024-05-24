use std::net::SocketAddr;

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR};
use http_client::HttpClient;
use http_server::{ApiRequest, ApiResponse, Request, Response, ResponseError};

use auth_server_http_proto::{UserNameForgotRequest, UserNameForgotResponse};
use gateway_http_proto::{
    UserNameForgotRequest as GatewayUserNameForgotRequest,
    UserNameForgotResponse as GatewayUserNameForgotResponse,
};

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
    let gateway_request = match GatewayUserNameForgotRequest::from_request(incoming_request) {
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

    let auth_request = UserNameForgotRequest::new(&gateway_request.email);

    http_server::log_util::send_req(host_name, auth_server, UserNameForgotRequest::name());
    match HttpClient::send(&auth_addr, auth_port, auth_request).await {
        Ok(_auth_response) => {
            http_server::log_util::recv_res(
                host_name,
                auth_server,
                UserNameForgotResponse::name(),
            );

            http_server::log_util::send_res(host_name, GatewayUserNameForgotResponse::name());
            return Ok(GatewayUserNameForgotResponse::new().to_response());
        }
        Err(e) => {
            http_server::log_util::recv_res(host_name, auth_server, "internal_server_error");
            http_server::log_util::send_res(host_name, "internal_server_error");
            return Err(ResponseError::InternalServerError(e.to_string()));
        }
    }
}
