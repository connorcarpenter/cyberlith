mod endpoints;
mod state;

use std::{net::SocketAddr, thread, time::Duration};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use config::{AUTH_SERVER_PORT, SELF_BINDING_ADDR};
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use crate::state::State;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Auth Server starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), AUTH_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::new()));

    endpoints::user_register(&mut server, state.clone());
    endpoints::user_register_confirm(&mut server, state.clone());
    endpoints::user_login(&mut server, state.clone());
    endpoints::user_name_forgot(&mut server, state.clone());
    endpoints::user_password_forgot(&mut server, state.clone());
    endpoints::user_password_reset(&mut server, state.clone());
    endpoints::token_validate(&mut server, state.clone());

    server.start();

    loop {
        thread::sleep(Duration::from_secs(5));
        info!(".");
    }
}
