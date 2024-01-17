use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::{events::ConnectEvent, Server};

pub fn connect_events(server: Server, mut event_reader: EventReader<ConnectEvent>) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);
    }
}
