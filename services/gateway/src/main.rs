mod endpoints;

use std::{net::SocketAddr, thread};

use config::{GATEWAY_PORT, SELF_BINDING_ADDR};
use http_server::Server;
use logging::info;

pub fn main() {
    logging::initialize();

    info!("Gateway starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), GATEWAY_PORT);

    let mut server = Server::new(socket_addr);

    // -> region

    // game client logs into session server
    endpoints::region::session_connect(&mut server);

    // -> auth

    // user registers for the first time
    endpoints::auth::user_register(&mut server);

    // user confirms registration
    endpoints::auth::user_register_confirm(&mut server);

    // user login
    endpoints::auth::user_login(&mut server);

    // refresh token grant
    endpoints::auth::refresh_token_grant(&mut server);

    // user name forgot
    endpoints::auth::user_name_forgot(&mut server);

    // user password forgot
    endpoints::auth::user_password_forgot(&mut server);

    // user password reset
    endpoints::auth::user_password_reset(&mut server);

    // -> content

    // user password reset
    server.serve_file("launcher.html");

    server.start();

    thread::park();

    info!("Shutting down...");
}
