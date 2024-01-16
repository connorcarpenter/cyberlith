use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::events::{DespawnEntityEvent, SpawnEntityEvent};

use crate::app::plugin::Main;

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent<Main>>) {
    for event in event_reader.read() {
        info!("entity: `{:?}`, spawned", event.entity);
    }
}

pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent<Main>>) {
    for event in event_reader.read() {
        info!("entity: `{:?}`, despawned", event.entity);
    }
}
