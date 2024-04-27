use std::net::SocketAddr;

use http_server::{Request, ResponseError, Response};

pub(crate) async fn handler(
    _addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {
    return Ok(Response::redirect(&incoming_request.url, "/game"));
}