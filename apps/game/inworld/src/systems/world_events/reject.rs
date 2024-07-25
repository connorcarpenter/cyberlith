use bevy_ecs::event::EventReader;

use game_engine::{world::WorldRejectEvent, logging::info};

pub fn reject_events(
    mut event_reader: EventReader<WorldRejectEvent>
) {
    for _ in event_reader.read() {
        info!("Client rejected from connecting to Server");
    }
}