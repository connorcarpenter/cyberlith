use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::events::{DespawnEntityEvent, SpawnEntityEvent};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(entity) in event_reader.iter() {
        info!("entity: `{:?}`, spawned", entity);
    }
}

pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent>) {
    for DespawnEntityEvent(entity) in event_reader.iter() {
        info!("entity: `{:?}`, despawned", entity);
    }
}
