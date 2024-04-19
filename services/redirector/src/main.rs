
use std::{net::SocketAddr, thread};

use config::{REDIRECTOR_PORT, PUBLIC_IP_ADDR, SELF_BINDING_ADDR, PUBLIC_PROTOCOL};
use http_server::{http_log_util, RedirectServer, Response};
use logging::info;

pub fn main() {
    logging::initialize();

    info!("Redirector starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), REDIRECTOR_PORT);

    let mut server = RedirectServer::new(socket_addr);

    server.endpoint(Box::new(|(socket_addr, request)| {
        Box::pin(async move {

            http_log_util::recv_req(
                "redirector",
                "client",
                format!("[{}][{} {}]", socket_addr, request.method.as_str(), request.url).as_str()
            );

            let response = Response::redirect(
                format!("{}://{}", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR).as_str(),
            );

            http_log_util::send_res(
                "redirector",
                "client",
                format!("redirect to {}", response.url).as_str(),
            );

            Ok(response)
        })
    }));

    server.start();

    thread::park();

    info!("Shutting down...");
}
