use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::{
    events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent},
    Server,
};

use cybl_nexus_proto::messages::Auth;

pub fn auth_events(mut server: Server, mut event_reader: EventReader<AuthEvents>) {
    for events in event_reader.iter() {
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
    for ConnectEvent(user_key) in event_reader.iter() {
        let address = server
            .user(user_key)
            // Get User's address for logging
            .address();

        info!("Server connected to: {}", address);
    }
}

pub fn disconnect_events(mut event_reader: EventReader<DisconnectEvent>) {
    for DisconnectEvent(_user_key, user) in event_reader.iter() {
        info!("Server disconnected from: {:?}", user.address);
    }
}

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.iter() {
        info!("Server Error: {:?}", error);
    }
}
