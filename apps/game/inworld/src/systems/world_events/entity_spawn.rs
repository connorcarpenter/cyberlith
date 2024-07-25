use bevy_ecs::event::EventReader;

use game_engine::{world::WorldSpawnEntityEvent, logging::info};

pub fn spawn_entity_events(
    mut event_reader: EventReader<WorldSpawnEntityEvent>
) {
    for event in event_reader.read() {
        info!(
            "received Spawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}