mod auth_handler;
mod demultiply_handler;
mod endpoints;
mod rate_limiter;
mod register_token;

use std::{net::SocketAddr, thread, time::Duration};

use config::{CONTENT_SERVER_PORT, CONTENT_SERVER_RECV_ADDR, GATEWAY_PORT, GATEWAY_SERVER_CPU_PRIORITY, PUBLIC_IP_ADDR, PUBLIC_PROTOCOL, SELF_BINDING_ADDR, SESSION_SERVER_RECV_ADDR, SESSION_SERVER_SIGNAL_PORT, TargetEnv, TOTAL_CPU_PRIORITY, WORLD_SERVER_RECV_ADDR, WORLD_SERVER_SIGNAL_PORT};
use endpoints::{redirect, session_connect};
use http_server::{ApiRequest, ApiServer, async_dup::Arc, executor, executor::smol, executor::smol::lock::RwLock, Method, ProxyServer, Server};
use logging::info;

use gateway_http_proto::{
    UserLoginRequest as GatewayUserLoginRequest,
    UserNameForgotRequest as GatewayUserNameForgotRequest,
    UserPasswordForgotRequest as GatewayUserPasswordForgotRequest,
    UserPasswordResetRequest as GatewayUserPasswordResetRequest,
    UserRegisterRequest as GatewayUserRegisterRequest,
};

pub fn main() {
    logging::initialize();
    executor::setup(GATEWAY_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    info!("Gateway starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), GATEWAY_PORT);

    let mut server = Server::new(socket_addr);

    let gateway = "gateway";
    let required_host_www = if TargetEnv::is_local() {
        None
    } else {
        Some((
            format!("{}", PUBLIC_IP_ADDR),
            format!("{}://{}", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR),
        ))
    };
    let required_host_www = required_host_www
        .as_ref()
        .map(|(s1, s2)| (s1.as_str(), Some(s2.as_str())));

    let api_allow_origin = if TargetEnv::is_local() {
        "*".to_string()
    } else {
        format!("{}://{}", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR)
    };
    let api_allow_origin = Some(api_allow_origin.as_str());

    // middleware

    // -> rate limiter
    let global_rate_limiter =
        rate_limiter::add_middleware(&mut server, 100, std::time::Duration::from_secs(8));

    // routes

    // -> auth
    {
        // user login
        server.raw_endpoint(
            gateway,
            required_host_www,
            api_allow_origin,
            GatewayUserLoginRequest::method(),
            GatewayUserLoginRequest::path(),
            endpoints::user_login::handler,
        );
        // user register
        server.raw_endpoint(
            gateway,
            required_host_www,
            api_allow_origin,
            GatewayUserRegisterRequest::method(),
            GatewayUserRegisterRequest::path(),
            endpoints::user_register::handler,
        );
        // user name forgot
        server.raw_endpoint(
            gateway,
            required_host_www,
            api_allow_origin,
            GatewayUserNameForgotRequest::method(),
            GatewayUserNameForgotRequest::path(),
            endpoints::user_name_forgot::handler,
        );
        // user password forgot
        server.raw_endpoint(
            gateway,
            required_host_www,
            api_allow_origin,
            GatewayUserPasswordForgotRequest::method(),
            GatewayUserPasswordForgotRequest::path(),
            endpoints::user_password_forgot::handler,
        );
        // user password reset
        server.raw_endpoint(
            gateway,
            required_host_www,
            api_allow_origin,
            GatewayUserPasswordResetRequest::method(),
            GatewayUserPasswordResetRequest::path(),
            endpoints::user_password_reset::handler,
        );
    }

    // -> session
    {
        let session_protocol = session_server_naia_proto::protocol();
        let session_protocol_endpoint = session_protocol.get_rtc_endpoint();
        let session_protocol = Arc::new(RwLock::new(session_protocol));

        server
            .raw_endpoint(
                gateway,
                required_host_www,
                api_allow_origin,
                Method::Post,
                &session_protocol_endpoint,
                move |addr, req| {
                    let protocol = session_protocol.clone();
                    async move { session_connect::handler(protocol, addr, req).await }
                },
            )
            .request_middleware(auth_handler::require_auth_tokens);

        let session_server = "session_server";
        let addr = SESSION_SERVER_RECV_ADDR;
        let port = SESSION_SERVER_SIGNAL_PORT.to_string();

        server.serve_proxy(
            gateway,
            required_host_www,
            api_allow_origin,
            Method::Options,
            &session_protocol_endpoint,
            session_server,
            addr,
            &port,
            &session_protocol_endpoint,
        );
    }

    // -> world
    {
        let world_server = "world_server";
        let addr = WORLD_SERVER_RECV_ADDR;
        let port = WORLD_SERVER_SIGNAL_PORT.to_string();

        let world_protocol_endpoint = world_server_naia_proto::protocol().get_rtc_endpoint();

        server
            .serve_proxy(
                gateway,
                required_host_www,
                api_allow_origin,
                Method::Post,
                &world_protocol_endpoint,
                world_server,
                addr,
                &port,
                &world_protocol_endpoint,
            )
            .request_middleware(auth_handler::require_auth_tokens);

        server.serve_proxy(
            gateway,
            required_host_www,
            api_allow_origin,
            Method::Options,
            &world_protocol_endpoint,
            world_server,
            addr,
            &port,
            &world_protocol_endpoint,
        );
    }

    // -> content
    {
        let content_server = "content_server";
        let addr = CONTENT_SERVER_RECV_ADDR;
        let port = CONTENT_SERVER_PORT.to_string();

        server
            .serve_proxy(
                gateway,
                required_host_www,
                None,
                Method::Get,
                "",
                content_server,
                addr,
                &port,
                "launcher.html",
            )
            .request_middleware(register_token::handle)
            .request_middleware(auth_handler::if_auth_tokens_and_offline_redirect_game);
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
        // used when hitting "/game"
        server
            .serve_proxy(
                gateway,
                required_host_www,
                None,
                Method::Get,
                "game",
                content_server,
                addr,
                &port,
                "game.html",
            )
            .request_middleware(auth_handler::require_auth_tokens_or_redirect_home)
            .request_middleware(demultiply_handler::require_offline_or_redirect_home);
        // used when hitting "/game.html"
        server.raw_endpoint(
            gateway,
            required_host_www,
            None,
            Method::Get,
            "game.html",
            redirect::redirect_to_game,
        );
        server
            .serve_proxy(
                gateway,
                required_host_www,
                None,
                Method::Get,
                "game.js",
                content_server,
                addr,
                &port,
                "game.js",
            )
            .request_middleware(auth_handler::require_auth_tokens);
        server
            .serve_proxy(
                gateway,
                required_host_www,
                None,
                Method::Get,
                "game_bg.wasm",
                content_server,
                addr,
                &port,
                "game_bg.wasm",
            )
            .request_middleware(auth_handler::require_auth_tokens);
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
        vec!["cyberlith.com".to_string(), "www.cyberlith.com".to_string()],
        vec!["admin@cyberlith.com".to_string()],
    ));
}

#[cfg(all(feature = "local", not(feature = "prod")))]
fn start_server(server: Server) {
    server.start();
}
