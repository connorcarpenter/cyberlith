use bevy_log::info;

use naia_bevy_server::{transport::webrtc, Server};

pub fn init(mut server: Server) {
    info!("World Server starting up");

    let server_addresses = webrtc::ServerAddrs::new(
        "127.0.0.1:14201"
            .parse()
            .expect("could not parse Signaling address/port"),
        // IP Address to listen on for UDP WebRTC data channels
        "127.0.0.1:14202"
            .parse()
            .expect("could not parse WebRTC data address/port"),
        // The public WebRTC IP address to advertise
        "http://127.0.0.1:14202",
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);
}
