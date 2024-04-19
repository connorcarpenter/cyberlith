mod endpoints;

use std::{net::SocketAddr, thread};

use config::{
    CONTENT_SERVER_PORT, CONTENT_SERVER_RECV_ADDR, GATEWAY_PORT, SELF_BINDING_ADDR,
    SESSION_SERVER_RECV_ADDR, SESSION_SERVER_SIGNAL_PORT, WORLD_SERVER_RECV_ADDR,
    WORLD_SERVER_SIGNAL_PORT,
};
use http_server::{Method, ProxyServer, Server};
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
        server.serve_proxy(
            gateway,
            Method::Get,
            "",
            content_server,
            addr,
            &port,
            "launcher.html",
        );
        server.serve_proxy(
            gateway,
            Method::Get,
            "launcher.js",
            content_server,
            addr,
            &port,
            "launcher.js",
        );
        server.serve_proxy(
            gateway,
            Method::Get,
            "launcher_bg.wasm",
            content_server,
            addr,
            &port,
            "launcher_bg.wasm",
        );
        server.serve_proxy(
            gateway,
            Method::Get,
            "game",
            content_server,
            addr,
            &port,
            "game.html",
        );
        server.serve_proxy(
            gateway,
            Method::Get,
            "game.js",
            content_server,
            addr,
            &port,
            "game.js",
        );
        server.serve_proxy(
            gateway,
            Method::Get,
            "game_bg.wasm",
            content_server,
            addr,
            &port,
            "game_bg.wasm",
        );
    }

    // -> session
    {
        let gateway = "gateway";
        let addr = SESSION_SERVER_RECV_ADDR;
        let port = SESSION_SERVER_SIGNAL_PORT.to_string();
        server.serve_proxy(
            gateway,
            Method::Post,
            "session_rtc",
            "session_server",
            addr,
            &port,
            "session_rtc",
        );
    }

    // -> world
    {
        let gateway = "gateway";
        let addr = WORLD_SERVER_RECV_ADDR;
        let port = WORLD_SERVER_SIGNAL_PORT.to_string();
        server.serve_proxy(
            gateway,
            Method::Post,
            "world_rtc",
            "world_server",
            addr,
            &port,
            "world_rtc",
        );
    }

    // start server

    start_server(server);

    thread::park();

    info!("Shutting down...");
}

#[cfg(all(feature = "prod", not(feature = "local")))]
fn start_server(server: Server) {
    use http_server::{acme::Config, HttpsServer};

    server.https_start(Config::new(
        true,
        vec![
            "cyberlith.com".to_string(),
            "www.cyberlith.com".to_string(),
            "api.cyberlith.com".to_string(),
        ],
        vec!["admin@cyberlith.com".to_string()],
    ));
}

#[cfg(all(feature = "local", not(feature = "prod")))]
fn start_server(server: Server) {
    server.start();
}
