
use std::{net::SocketAddr, thread};

use config::{AUTH_SERVER_PORT, PUBLIC_IP_ADDR, AUTH_SERVER_RECV_ADDR, SUBDOMAIN_WWW, SUBDOMAIN_API, CONTENT_SERVER_PORT, CONTENT_SERVER_RECV_ADDR, GATEWAY_PORT, REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SELF_BINDING_ADDR, SESSION_SERVER_RECV_ADDR, SESSION_SERVER_SIGNAL_PORT, WORLD_SERVER_RECV_ADDR, WORLD_SERVER_SIGNAL_PORT};
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
    let required_host_api = if SUBDOMAIN_API.is_empty() {
        None
    } else {
        Some(format!("{}.{}", SUBDOMAIN_API, PUBLIC_IP_ADDR))
    };
    let required_host_api = required_host_api.as_ref().map(|s| s.as_str());
    let required_host_www = if SUBDOMAIN_WWW.is_empty() {
        None
    } else {
        Some(format!("{}.{}", SUBDOMAIN_WWW, PUBLIC_IP_ADDR))
    };
    let required_host_www = required_host_www.as_ref().map(|s| s.as_str());

    // -> region
    {
        let auth_server = "region_server";
        let addr = REGION_SERVER_RECV_ADDR;
        let port = REGION_SERVER_PORT.to_string();

        // session connect
        server.serve_api_proxy::<SessionConnectRequest>(
            gateway,
            required_host_api,
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
            required_host_api,
            auth_server,
            addr,
            &port,
        );
        // user register
        server.serve_api_proxy::<UserRegisterRequest>(
            gateway,
            required_host_api,
            auth_server,
            addr,
            &port,
        );
        // user register confirm
        server.serve_api_proxy::<UserRegisterConfirmRequest>(
            gateway,
            required_host_api,
            auth_server,
            addr,
            &port,
        );
        // refresh token grant
        server.serve_api_proxy::<RefreshTokenGrantRequest>(
            gateway,
            required_host_api,
            auth_server,
            addr,
            &port,
        );
        // user name forgot
        server.serve_api_proxy::<UserNameForgotRequest>(
            gateway,
            required_host_api,
            auth_server,
            addr,
            &port,
        );
        // user password forgot
        server.serve_api_proxy::<UserPasswordForgotRequest>(
            gateway,
            required_host_api,
            auth_server,
            addr,
            &port,
        );
        // user password reset
        server.serve_api_proxy::<UserPasswordResetRequest>(
            gateway,
            required_host_api,
            auth_server,
            addr,
            &port,
        );
    }

    // -> session
    {
        let session_server = "session_server";
        let addr = SESSION_SERVER_RECV_ADDR;
        let port = SESSION_SERVER_SIGNAL_PORT.to_string();

        server.serve_proxy(
            gateway,
            required_host_api,
            Method::Post,
            "session_rtc",
            session_server,
            addr,
            &port,
            "session_rtc",
        );
    }

    // -> world
    {
        let world_server = "world_server";
        let addr = WORLD_SERVER_RECV_ADDR;
        let port = WORLD_SERVER_SIGNAL_PORT.to_string();

        server.serve_proxy(
            gateway,
            required_host_api,
            Method::Post,
            "world_rtc",
            world_server,
            addr,
            &port,
            "world_rtc",
        );
    }

    // -> content
    {
        let content_server = "content_server";
        let addr = CONTENT_SERVER_RECV_ADDR;
        let port = CONTENT_SERVER_PORT.to_string();

        server.serve_proxy(
            gateway,
            required_host_www,
            Method::Get,
            "",
            content_server,
            addr,
            &port,
            "launcher.html",
        );
        server.serve_proxy(
            gateway,
            required_host_www,
            Method::Get,
            "launcher.js",
            content_server,
            addr,
            &port,
            "launcher.js",
        );
        server.serve_proxy(
            gateway,
            required_host_www,
            Method::Get,
            "launcher_bg.wasm",
            content_server,
            addr,
            &port,
            "launcher_bg.wasm",
        );
        server.serve_proxy(
            gateway,
            required_host_www,
            Method::Get,
            "game",
            content_server,
            addr,
            &port,
            "game.html",
        );
        server.serve_proxy(
            gateway,
            required_host_www,
            Method::Get,
            "game.js",
            content_server,
            addr,
            &port,
            "game.js",
        );
        server.serve_proxy(
            gateway,
            required_host_www,
            Method::Get,
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
