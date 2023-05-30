use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::events::{
    DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
    UpdateComponentEvents,
};

use vortex_proto::components::{FileSystemChild, FileSystemEntry, FileSystemRootChild};

pub fn spawn_entity_events(event_reader: EventReader<SpawnEntityEvent>) {
    // unused for now
    // for SpawnEntityEvent(user_key, entity) in event_reader.iter() {
    //     info!("spawned entity");
    // }
}

pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent>) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        info!("despawned entity");
    }
}

pub fn insert_component_events(mut event_reader: EventReader<InsertComponentEvents>) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {}

        // on FileSystemRootChild Insert Event
        for (user_key, entity) in events.read::<FileSystemRootChild>() {}

        // on FileSystemChild Insert Event
        for (user_key, entity) in events.read::<FileSystemChild>() {}
    }
}

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (user_key, entity, component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");
        }
        for (user_key, entity, component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");
        }
    }
}

pub fn update_component_events(mut event_reader: EventReader<UpdateComponentEvents>) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {}
        // on FileSystemChild Update Event
        for (user_key, entity) in events.read::<FileSystemChild>() {}
    }
}
