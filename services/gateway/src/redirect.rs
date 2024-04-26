use std::net::SocketAddr;

use http_client::ResponseError;
use http_server::{Request, Response};

pub(crate) async fn game_html_redirect_handler(
    args: (SocketAddr, Request),
) -> Result<Response, ResponseError> {
    let (_addr, incoming_request) = args;
    return Ok(Response::redirect(&incoming_request.url, "/game"));
}