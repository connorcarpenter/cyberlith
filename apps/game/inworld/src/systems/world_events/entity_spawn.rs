use bevy_ecs::event::EventReader;

use game_engine::logging::info;
use game_app_network::world::WorldSpawnEntityEvent;

pub fn spawn_entity_events(mut event_reader: EventReader<WorldSpawnEntityEvent>) {
    for event in event_reader.read() {
        info!(
            "received Spawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}
