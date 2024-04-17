mod endpoints;

use std::{net::SocketAddr, thread};

use config::{GATEWAY_PORT, CONTENT_SERVER_RECV_ADDR, CONTENT_SERVER_PORT, SELF_BINDING_ADDR};
use http_server::{RemoteFileServer, Server};
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

    {
        let gateway = "gateway";
        let content_server = "content_server";
        let addr = CONTENT_SERVER_RECV_ADDR;
        let port = CONTENT_SERVER_PORT.to_string();
        server.serve_remote_file(
            gateway, "",
            content_server, addr, &port, "launcher.html"
        );
        server.serve_remote_file(
            gateway, "launcher.js",
            content_server, addr, &port, "launcher.js"
        );
        server.serve_remote_file(
            gateway, "launcher_bg.wasm",
            content_server, addr, &port, "launcher_bg.wasm"
        );
        server.serve_remote_file(
            gateway, "game",
            content_server, addr, &port, "game.html"
        );
        server.serve_remote_file(
            gateway, "game.js",
            content_server, addr, &port, "game.js"
        );
        server.serve_remote_file(
            gateway, "game_bg.wasm",
            content_server, addr, &port, "game_bg.wasm"
        );
    }

    // start server

    server.start();

    thread::park();

    info!("Shutting down...");
}
