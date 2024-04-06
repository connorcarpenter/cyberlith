mod endpoints;

use std::{net::SocketAddr, time::Duration};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use config::{GATEWAY_PORT, SELF_BINDING_ADDR};
use http_server::Server;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Gateway starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), GATEWAY_PORT);

    let mut server = Server::new(socket_addr);

    // game client logs into session server
    endpoints::region::session_connect(&mut server);

    // user registers for the first time
    endpoints::auth::user_register(&mut server);

    // user confirms registration
    endpoints::auth::user_register_confirm(&mut server);

    // user login
    endpoints::auth::user_login(&mut server);

    // user name forgot
    endpoints::auth::user_name_forgot(&mut server);

    // user password forgot
    endpoints::auth::user_password_forgot(&mut server);

    // user password reset
    endpoints::auth::user_password_reset(&mut server);

    server.start();

    loop {
        std::thread::sleep(Duration::from_secs(5));
        info!(".");
    }
}
