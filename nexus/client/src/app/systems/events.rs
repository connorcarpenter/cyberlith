use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::{
    events::{ConnectEvent, DisconnectEvent, ErrorEvent, RejectEvent},
    Client,
};

pub fn connect_events(client: Client, mut event_reader: EventReader<ConnectEvent>) {
    for _ in event_reader.iter() {
        let server_address = client.server_address().unwrap();
        info!("Client connected to: {}", server_address);
    }
}

pub fn reject_events(mut event_reader: EventReader<RejectEvent>) {
    for _ in event_reader.iter() {
        info!("Client rejected from connecting to Server");
    }
}

pub fn disconnect_events(mut event_reader: EventReader<DisconnectEvent>) {
    for _ in event_reader.iter() {
        info!("Client disconnected from Server");
    }
}

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.iter() {
        info!("Client Error: {:?}", error);
    }
}
