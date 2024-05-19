
mod region_connection;
mod session_connections;
mod state;

use std::{net::SocketAddr, thread, time::Duration};

use config::{SOCIAL_SERVER_PORT, SELF_BINDING_ADDR};
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, Server};
use logging::info;

use crate::state::State;

pub fn main() {
    logging::initialize();

    // setup state
    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(16);
    let state = Arc::new(RwLock::new(State::new(
        registration_resend_rate,
        region_server_disconnect_timeout,
    )));

    // setup listening http server
    info!("Social Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), SOCIAL_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let server_name = "social_server";

    region_connection::recv_heartbeat_request(server_name, &mut server, state.clone());

    session_connections::recv_connect_session_server_request(server_name, &mut server, state.clone());
    session_connections::recv_disconnect_session_server_request(server_name, &mut server, state.clone());

    server.start();

    // send registration
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            region_connection::send_register_instance_request(state_clone_2).await;
            thread::sleep(Duration::from_secs(5));
        }
    });

    // handle disconnection
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            region_connection::process_region_server_disconnect(state_clone_2).await;
            thread::sleep(Duration::from_secs(5));
        }
    });

    thread::park();

    info!("Shutting down...");
}
