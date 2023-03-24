use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::events::DisconnectEvent;

pub fn disconnect_events(mut event_reader: EventReader<DisconnectEvent>) {
    for _ in event_reader.iter() {
        info!("Client disconnected from Server");
    }
}
