mod session_connect;
mod redirect;
mod rate_limiter;
mod access_token_checker;
mod user_login;
mod target_env;
mod world_connect;

use std::{time::Duration, net::SocketAddr, thread};

use config::{
    AUTH_SERVER_PORT, AUTH_SERVER_RECV_ADDR, CONTENT_SERVER_PORT, CONTENT_SERVER_RECV_ADDR,
    GATEWAY_PORT, PUBLIC_IP_ADDR, PUBLIC_PROTOCOL, SELF_BINDING_ADDR, SUBDOMAIN_API, SUBDOMAIN_WWW,
    WORLD_SERVER_RECV_ADDR, WORLD_SERVER_SIGNAL_PORT, SESSION_SERVER_RECV_ADDR, SESSION_SERVER_SIGNAL_PORT,
};
use http_server::{smol::lock::RwLock, async_dup::Arc, ApiRequest, ApiServer, Method, ProxyServer, Server, smol};
use logging::info;

use auth_server_http_proto::{
    RefreshTokenGrantRequest, UserLoginRequest, UserNameForgotRequest, UserPasswordForgotRequest,
    UserPasswordResetRequest, UserRegisterConfirmRequest, UserRegisterRequest,
};

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
    let required_host_api = required_host_api.as_ref().map(|s| (s.as_str(), None));
    let required_host_www = if SUBDOMAIN_WWW.is_empty() {
        None
    } else {
        Some(format!("{}.{}", SUBDOMAIN_WWW, PUBLIC_IP_ADDR))
    };
    let rd = format!("{}://{}.{}", PUBLIC_PROTOCOL, SUBDOMAIN_WWW, PUBLIC_IP_ADDR);
    let required_host_www = required_host_www
        .as_ref()
        .map(|s| (s.as_str(), Some(rd.as_str())));

    let api_allow_origin = if SUBDOMAIN_API.is_empty() {
        "*".to_string()
    } else {
        format!("{}://{}.{}", PUBLIC_PROTOCOL, SUBDOMAIN_WWW, PUBLIC_IP_ADDR)
    };
    let api_allow_origin = Some(api_allow_origin.as_str());

    // middleware

    // -> rate limiter
    let global_rate_limiter = rate_limiter::add_middleware(&mut server, 100, std::time::Duration::from_secs(8));

    // routes

    // -> auth
    {
        let auth_server = "auth_server";
        let addr = AUTH_SERVER_RECV_ADDR;
        let port = AUTH_SERVER_PORT.to_string();

        // user login
        server.raw_endpoint(
            gateway,
            required_host_www, // uses this to set the cookie appropriately
            api_allow_origin,
            UserLoginRequest::method(),
            UserLoginRequest::path(),
            user_login::handler,
        );
        // user register
        server.serve_api_proxy::<UserRegisterRequest>(
            gateway,
            required_host_api,
            api_allow_origin,
            auth_server,
            addr,
            &port,
        );
        // user register confirm
        server.serve_api_proxy::<UserRegisterConfirmRequest>(
            gateway,
            required_host_api,
            api_allow_origin,
            auth_server,
            addr,
            &port,
        );
        // refresh token grant
        server.serve_api_proxy::<RefreshTokenGrantRequest>(
            gateway,
            required_host_api,
            api_allow_origin,
            auth_server,
            addr,
            &port,
        );
        // user name forgot
        server.serve_api_proxy::<UserNameForgotRequest>(
            gateway,
            required_host_api,
            api_allow_origin,
            auth_server,
            addr,
            &port,
        );
        // user password forgot
        server.serve_api_proxy::<UserPasswordForgotRequest>(
            gateway,
            required_host_api,
            api_allow_origin,
            auth_server,
            addr,
            &port,
        );
        // user password reset
        server.serve_api_proxy::<UserPasswordResetRequest>(
            gateway,
            required_host_api,
            api_allow_origin,
            auth_server,
            addr,
            &port,
        );
    }

    // -> session
    {
        let session_protocol_1 = Arc::new(RwLock::new(session_server_naia_proto::protocol()));
        let session_protocol_2 = session_protocol_1.clone();

        server.raw_endpoint(
            gateway,
            required_host_api,
            api_allow_origin,
            Method::Post,
            "session_rtc",
            move |addr, req| {
                let protocol = session_protocol_1.clone();
                async move { session_connect::handler(protocol, addr, req).await }
            },
        ).middleware(move |addr, req| {
            let protocol = session_protocol_2.clone();
            async move { session_connect::auth_middleware(protocol, addr, req).await }
        });

        let session_server = "session_server";
        let addr = SESSION_SERVER_RECV_ADDR;
        let port = SESSION_SERVER_SIGNAL_PORT.to_string();

        server.serve_proxy(
            gateway,
            required_host_api,
            api_allow_origin,
            Method::Options,
            "session_rtc",
            session_server,
            addr,
            &port,
            "session_rtc",
        );
    }

    // -> world
    {
        let world_protocol_1 = Arc::new(RwLock::new(world_server_naia_proto::protocol()));
        let world_protocol_2 = world_protocol_1.clone();

        server.raw_endpoint(
            gateway,
            required_host_api,
            api_allow_origin,
            Method::Post,
            "world_rtc",
            move |addr, req| {
                let protocol = world_protocol_1.clone();
                async move { world_connect::handler(protocol, addr, req).await }
            },
        ).middleware(move |addr, req| {
            let protocol = world_protocol_2.clone();
            async move { world_connect::auth_middleware(protocol, addr, req).await }
        });

        let world_server = "world_server";
        let addr = WORLD_SERVER_RECV_ADDR;
        let port = WORLD_SERVER_SIGNAL_PORT.to_string();

        server.serve_proxy(
            gateway,
            required_host_api,
            api_allow_origin,
            Method::Options,
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
            None,
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
            None,
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
            None,
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
            None,
            Method::Get,
            "game",
            content_server,
            addr,
            &port,
            "game.html",
        ).middleware(access_token_checker::www_middleware);
        server.raw_endpoint(
            gateway,
            required_host_www,
            None,
            Method::Get,
            "game.html",
            redirect::handler,
        ).middleware(access_token_checker::www_middleware);
        server.serve_proxy(
            gateway,
            required_host_www,
            None,
            Method::Get,
            "game.js",
            content_server,
            addr,
            &port,
            "game.js",
        ).middleware(access_token_checker::www_middleware);
        server.serve_proxy(
            gateway,
            required_host_www,
            None,
            Method::Get,
            "game_bg.wasm",
            content_server,
            addr,
            &port,
            "game_bg.wasm",
        ).middleware(access_token_checker::www_middleware);
    }

    // prune expired rate limiter entries
    Server::spawn(async move {
        loop {
            smol::Timer::after(Duration::from_secs(60 * 60)).await;

            let mut global_rate_limiter = global_rate_limiter.write().await;
            global_rate_limiter.prune().await;
        }
    });

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
