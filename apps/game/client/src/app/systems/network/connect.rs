use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::{events::ConnectEvent, Client};

pub fn connect_events(client: Client, mut event_reader: EventReader<ConnectEvent>) {
    for _ in event_reader.read() {
        let server_address = client.server_address().unwrap();
        info!("Client connected to: {}", server_address);
    }
}
