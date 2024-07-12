mod server;

use std::{net::SocketAddr, thread};

use config::{REDIRECTOR_SERVER_CPU_PRIORITY, REDIRECTOR_PORT, SELF_BINDING_ADDR, TOTAL_CPU_PRIORITY};
use logging::info;

use crate::server::RedirectServer;

pub fn main() {
    logging::initialize();
    executor::setup(REDIRECTOR_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    info!("Redirector starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), REDIRECTOR_PORT);

    RedirectServer::start(socket_addr);

    thread::park();

    info!("Shutting down...");
}
