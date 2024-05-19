use std::net::SocketAddr;

use http_server::{Request, Response, ResponseError};

pub(crate) async fn redirect_to_game(
    _addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {
    return Ok(Response::redirect(&incoming_request.url, "/game"));
}
