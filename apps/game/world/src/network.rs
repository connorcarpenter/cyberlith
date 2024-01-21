use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::{transport::webrtc, Server, events::{AuthEvents, DisconnectEvent, ErrorEvent, ConnectEvent}};

use world_proto::messages::Auth;

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

pub fn auth_events(mut server: Server, mut event_reader: EventReader<AuthEvents>) {
    for events in event_reader.read() {
        for (user_key, auth) in events.read::<Auth>() {
            if auth.username == "charlie" && auth.password == "12345" {
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
