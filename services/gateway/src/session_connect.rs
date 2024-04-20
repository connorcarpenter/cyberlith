use std::{net::SocketAddr};
use config::{SESSION_SERVER_RECV_ADDR, SESSION_SERVER_SIGNAL_PORT};

use http_client::ResponseError;
use http_server::{Request, Response};

pub(crate) async fn session_rtc_endpoint_handler(
    args: (SocketAddr, Request),
) -> Result<Response, ResponseError> {
    let (_addr, incoming_request) = args;

    let session_server = "session_server";
    let addr = SESSION_SERVER_RECV_ADDR;
    let port = SESSION_SERVER_SIGNAL_PORT.to_string();

    Ok(Response::ok(&incoming_request.url))
}
