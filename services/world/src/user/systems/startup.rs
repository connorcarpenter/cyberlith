use std::net::SocketAddr;

use naia_bevy_server::{transport::webrtc, Server};

use config::{
    PUBLIC_IP_ADDR, PUBLIC_PROTOCOL, SELF_BINDING_ADDR, WORLD_SERVER_SIGNAL_PORT,
    WORLD_SERVER_WEBRTC_PORT,
};
use logging::info;

pub fn server(mut server: Server) {
    info!("World Naia Server starting up");

    // set up server
    let server_addresses = webrtc::ServerAddrs::new(
        // IP Address to listen on for WebRTC signaling
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_SIGNAL_PORT),
        // IP Address to listen on for UDP WebRTC data channels
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_WEBRTC_PORT),
        // The public WebRTC IP address to advertise
        format!(
            "{}://{}:{}",
            PUBLIC_PROTOCOL, PUBLIC_IP_ADDR, WORLD_SERVER_WEBRTC_PORT
        )
        .as_str(),
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);
}
