use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Query, Res},
};
use bevy_log::info;

use naia_bevy_client::{
    events::{
        DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
        UpdateComponentEvents,
    },
    Client,
};

use vortex_proto::components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    resources::global::Global,
};

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
    mut commands: Commands,
    client: Client,
    global: Res<Global>,
    mut event_reader: EventReader<InsertComponentEvents>,
    mut parent_query: Query<&mut FileSystemParent>,
    child_query: Query<&FileSystemChild>,
    entry_query: Query<&FileSystemEntry>,
) {
    let project_root_entity = global.project_root_entity;
    let mut recent_parents: Option<HashMap<Entity, FileSystemParent>> = None;

    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for entry_entity in events.read::<FileSystemEntry>() {
            let entry = entry_query.get(entry_entity).unwrap();

            // Add FileSystemParent to directories
            if *entry.kind == EntryKind::Directory {
                if recent_parents.is_none() {
                    recent_parents = Some(HashMap::new());
                }
                let map = recent_parents.as_mut().unwrap();
                map.insert(entry_entity, FileSystemParent::new());
            }
            // Add FileSystemUiState to all entities
            commands
                .entity(entry_entity)
                .insert(FileSystemUiState::new());
        }

        // on FileSystemRootChild Insert Event
        for child_entity in events.read::<FileSystemRootChild>() {
            // Add children to root parent
            let entry = entry_query.get(child_entity).unwrap();
            let entry_kind = (*(entry.kind)).clone();
            let child_name = (*(entry.name)).clone();
            let mut parent = parent_query.get_mut(project_root_entity).unwrap();
            parent.add_child(entry_kind, child_name, child_entity);
        }
        // on FileSystemChild Insert Event
        for child_entity in events.read::<FileSystemChild>() {
            // Add children to directories
            let parent_entity_opt = child_query
                .get(child_entity)
                .unwrap()
                .parent_id
                .get(&client);
            let Some(parent_entity) = parent_entity_opt else {
                panic!("FileSystemChild component has no parent_id");
            };
            let entry = entry_query.get(child_entity).unwrap();
            let entry_kind = (*(entry.kind)).clone();
            let child_name = (*(entry.name)).clone();
            if let Ok(mut parent) = parent_query.get_mut(parent_entity) {
                parent.add_child(entry_kind, child_name, child_entity);
            } else {
                let Some(parent_map) = recent_parents.as_mut() else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", child_entity, parent_entity);
                };
                let Some(parent) = parent_map.get_mut(&parent_entity) else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", child_entity, parent_entity);
                };
                parent.add_child(entry_kind, child_name, child_entity);
            };
        }
        // Add all parents now that the children were able to process
        // Note that we do it this way because Commands aren't flushed till the end of the system
        if let Some(parent_map) = recent_parents.as_mut() {
            for (entity, parent) in parent_map.drain() {
                commands.entity(entity).insert(parent);
            }
        }
    }
}

pub fn update_component_events(
    //mut commands: Commands,
    //client: Client,
    //global: Res<Global>,
    mut event_reader: EventReader<UpdateComponentEvents>,
    //mut parent_query: Query<&mut FileSystemParent>,
    //child_query: Query<&FileSystemChild>,
    entry_query: Query<&FileSystemEntry>,
) {
    //let project_root_entity = global.project_root_entity;
    //let mut recent_parents: Option<HashMap<Entity, FileSystemParent>> = None;

    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_, entry_entity) in events.read::<FileSystemEntry>() {
            let entry = entry_query.get(entry_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemEntry: `{:?}` ({:?})",
                entry_entity, entry_name
            );
        }
        // on FileSystemRootChild Update Event
        for (_, child_entity) in events.read::<FileSystemRootChild>() {
            let entry = entry_query.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemRootChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on FileSystemChild Update Event
        for (_, child_entity) in events.read::<FileSystemChild>() {
            let entry = entry_query.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
    }
}

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (_entity, _component) in events.read::<FileSystemEntry>() {
            info!("removed FileSystemEntry component from entity");
            todo!();
        }
        for (_entity, _component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");
            todo!();
        }
        for (_entity, _component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");
            todo!();
        }
    }
}
