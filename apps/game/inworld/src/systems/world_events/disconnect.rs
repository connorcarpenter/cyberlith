use bevy_ecs::event::EventReader;

use game_engine::logging::info;
use game_app_network::world::WorldDisconnectEvent;

pub fn disconnect_events(mut event_reader: EventReader<WorldDisconnectEvent>) {
    for _ in event_reader.read() {
        info!("Client disconnected from Server");
    }
}
