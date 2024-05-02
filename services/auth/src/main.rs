mod emails;
mod endpoints;
mod error;
mod state;
mod types;
mod expire_manager;

use std::{net::SocketAddr, thread};
use std::time::Duration;

use config::{AUTH_SERVER_PORT, SELF_BINDING_ADDR};
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, Server, executor::smol};
use logging::info;

use crate::state::State;

pub fn main() {
    logging::initialize();

    info!("Auth Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), AUTH_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::new()));
    let server_name = "auth_server";

    endpoints::user_login(server_name, &mut server, state.clone());
    endpoints::user_register(server_name, &mut server, state.clone());
    endpoints::user_register_confirm(server_name, &mut server, state.clone());
    endpoints::user_name_forgot(server_name, &mut server, state.clone());
    endpoints::user_password_forgot(server_name, &mut server, state.clone());
    endpoints::user_password_reset(server_name, &mut server, state.clone());
    endpoints::access_token_validate(server_name, &mut server, state.clone());
    endpoints::refresh_token_grant(server_name, &mut server, state.clone());

    // expire tokens
    Server::spawn(async move {
        let state = state.clone();
        loop {
            smol::Timer::after(Duration::from_secs(60 * 15)).await; // 15 minutes

            let mut state = state.write().await;
            state.clear_expired_tokens();
        }
    });

    server.start();

    thread::park();

    info!("Shutting down...");
}
