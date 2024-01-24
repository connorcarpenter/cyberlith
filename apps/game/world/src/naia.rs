use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent},
    transport::webrtc,
    Server,
};

use world_server_naia_proto::messages::Auth;
use config::{WORLD_SERVER_SIGNAL_ADDR, WORLD_SERVER_WEBRTC_ADDR};

pub fn init(mut server: Server) {
    info!("World Naia Server starting up");

    let server_addresses = webrtc::ServerAddrs::new(
        WORLD_SERVER_SIGNAL_ADDR
            .parse()
            .expect("could not parse Signaling address/port"),
        // IP Address to listen on for UDP WebRTC data channels
        WORLD_SERVER_WEBRTC_ADDR
            .parse()
            .expect("could not parse WebRTC data address/port"),
        // The public WebRTC IP address to advertise
        format!("http://{}", WORLD_SERVER_WEBRTC_ADDR).as_str(),
    );
    let socket = webrtc::Socket::new(&server_addresses, server.socket_config());
    server.listen(socket);
}

pub fn auth_events(mut server: Server, mut event_reader: EventReader<AuthEvents>) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if auth.token == "the_login_token" {
                // Accept incoming connection
                server.accept_connection(&user_key);
            } else {
                // Reject incoming connection
                server.reject_connection(&user_key);
            }
        }
    }
}

pub fn connect_events(server: Server, mut event_reader: EventReader<ConnectEvent>) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);
    }
}

pub fn disconnect_events(mut event_reader: EventReader<DisconnectEvent>) {
    for DisconnectEvent(_user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address);
    }
}

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.read() {
        info!("Server Error: {:?}", error);
    }
}
