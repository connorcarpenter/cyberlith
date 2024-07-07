mod global_chat;
mod match_lobbies;
mod region;
mod session_servers;
mod state;
mod users;

use std::{net::SocketAddr, thread, time::Duration};

use config::{SELF_BINDING_ADDR, SOCIAL_SERVER_PORT};
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
    let host = "social";

    region::recv_heartbeat_request(host, &mut server, state.clone());

    session_servers::recv_connect_session_server_request(host, &mut server, state.clone());
    session_servers::recv_disconnect_session_server_request(host, &mut server, state.clone());

    users::recv_user_connected_request(host, &mut server, state.clone());
    users::recv_user_disconnected_request(host, &mut server, state.clone());
    users::recv_user_is_online_request(host, &mut server, state.clone());

    match_lobbies::recv_match_lobby_create_request(host, &mut server, state.clone());
    match_lobbies::recv_match_lobby_join_request(host, &mut server, state.clone());
    match_lobbies::recv_match_lobby_leave_request(host, &mut server, state.clone());
    match_lobbies::recv_match_lobby_send_message_request(host, &mut server, state.clone());

    global_chat::recv_global_chat_send_message_request(host, &mut server, state.clone());

    server.start();

    region::start_processes(state.clone());
    session_servers::start_processes(state.clone());

    thread::park();

    info!("Shutting down...");
}
