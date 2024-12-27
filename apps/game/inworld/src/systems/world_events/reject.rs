use bevy_ecs::event::EventReader;

use game_engine::logging::info;
use game_app_network::world::WorldRejectEvent;

pub fn reject_events(mut event_reader: EventReader<WorldRejectEvent>) {
    for _ in event_reader.read() {
        info!("Client rejected from connecting to Server");
    }
}
