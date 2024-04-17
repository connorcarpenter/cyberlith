mod endpoints;

use std::{str::FromStr, path::PathBuf, net::SocketAddr, thread};

use config::{CONTENT_SERVER_PORT, CONTENT_SERVER_RECV_ADDR, GATEWAY_PORT, SELF_BINDING_ADDR};
use http_server::{acme::Config, HttpsServer, RemoteFileServer, Server};
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
        server.serve_remote_file(gateway, "", content_server, addr, &port, "launcher.html");
        server.serve_remote_file(
            gateway,
            "launcher.js",
            content_server,
            addr,
            &port,
            "launcher.js",
        );
        server.serve_remote_file(
            gateway,
            "launcher_bg.wasm",
            content_server,
            addr,
            &port,
            "launcher_bg.wasm",
        );
        server.serve_remote_file(gateway, "game", content_server, addr, &port, "game.html");
        server.serve_remote_file(gateway, "game.js", content_server, addr, &port, "game.js");
        server.serve_remote_file(
            gateway,
            "game_bg.wasm",
            content_server,
            addr,
            &port,
            "game_bg.wasm",
        );
    }

    // start server

    start_server(server);

    thread::park();

    info!("Shutting down...");
}

#[cfg(all(feature = "prod", not(feature = "local")))]
fn start_server(mut server: Server) {
    server.https_start(
        Config::new(
            true,
            vec!["cyberlith.com".to_string(), "www.cyberlith.com".to_string()],
            vec!["admin@cyberlith.com".to_string()],
            Some(PathBuf::from_str("./acme_cache").unwrap())
        )
    );
}

#[cfg(all(feature = "local", not(feature = "prod")))]
fn start_server(mut server: Server) {
    server.start();
}
