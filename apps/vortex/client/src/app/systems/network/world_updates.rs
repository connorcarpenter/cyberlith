use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query},
};
use bevy_log::info;
use naia_bevy_client::events::{
    DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
};
use vortex_proto::components::FileSystemEntry;

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(_entity) in event_reader.iter() {
        info!("spawned entity");
    }
}

pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent>) {
    for DespawnEntityEvent(_entity) in event_reader.iter() {
        info!("despawned entity");
    }
}

pub fn insert_component_events(
    mut event_reader: EventReader<InsertComponentEvents>,
    entry_query: Query<&FileSystemEntry>,
) {
    for events in event_reader.iter() {
        for entity in events.read::<FileSystemEntry>() {
            let name: &str = entry_query.get(entity).unwrap().name.as_str();
            info!("add FileSystemEntry component: `{}` to entity", name);
        }
    }
}

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (_entity, _component) in events.read::<FileSystemEntry>() {
            info!("removed FileSystemEntry component from entity");
        }
    }
}
