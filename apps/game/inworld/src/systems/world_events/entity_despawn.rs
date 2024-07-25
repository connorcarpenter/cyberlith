use bevy_ecs::event::EventReader;

use game_engine::{world::WorldDespawnEntityEvent, logging::info};

pub fn despawn_entity_events(
    mut event_reader: EventReader<WorldDespawnEntityEvent>
) {
    for event in event_reader.read() {
        info!(
            "received Despawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}