use bevy_ecs::event::EventReader;

use game_engine::logging::info;
use game_app_network::world::WorldDespawnEntityEvent;

pub fn despawn_entity_events(mut event_reader: EventReader<WorldDespawnEntityEvent>) {
    for event in event_reader.read() {
        info!(
            "received Despawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}
