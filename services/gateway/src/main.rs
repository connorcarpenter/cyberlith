
use std::{net::SocketAddr, thread};

use config::{AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, CONTENT_SERVER_PORT, CONTENT_SERVER_RECV_ADDR, GATEWAY_PORT, REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SELF_BINDING_ADDR, SESSION_SERVER_RECV_ADDR, SESSION_SERVER_SIGNAL_PORT, WORLD_SERVER_RECV_ADDR, WORLD_SERVER_SIGNAL_PORT};
use http_server::{Method, ProxyServer, Server};
use logging::info;

use region_server_http_proto::SessionConnectRequest;
use auth_server_http_proto::{RefreshTokenGrantRequest, UserLoginRequest, UserNameForgotRequest, UserPasswordForgotRequest, UserPasswordResetRequest, UserRegisterConfirmRequest, UserRegisterRequest};

pub fn main() {
    logging::initialize();

    info!("Gateway starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), GATEWAY_PORT);

    let mut server = Server::new(socket_addr);

    let gateway = "gateway";

    // -> region
    {
        let auth_server = "region_server";
        let addr = REGION_SERVER_RECV_ADDR;
        let port = REGION_SERVER_PORT.to_string();

        // session connect
        server.serve_api_proxy::<SessionConnectRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
    }

    // -> auth
    {
        let auth_server = "auth_server";
        let addr = AUTH_SERVER_RECV_ADDR;
        let port = AUTH_SERVER_PORT.to_string();

        // user login
        server.serve_api_proxy::<UserLoginRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
        // user register
        server.serve_api_proxy::<UserRegisterRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
        // user register confirm
        server.serve_api_proxy::<UserRegisterConfirmRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
        // refresh token grant
        server.serve_api_proxy::<RefreshTokenGrantRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
        // user name forgot
        server.serve_api_proxy::<UserNameForgotRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
        // user password forgot
        server.serve_api_proxy::<UserPasswordForgotRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
        // user password reset
        server.serve_api_proxy::<UserPasswordResetRequest>(
            gateway,
            auth_server,
            addr,
            &port,
        );
    }

    // -> content
    {
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
