use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Query, ResMut},
};
use bevy_log::info;
use naia_bevy_client::{Client, events::{
    DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
    UpdateComponentEvents,
}};

use vortex_proto::components::{ChangelistEntry, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState}, resources::global::Global,
    systems::file_post_process,
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
    mut global: ResMut<Global>,
    mut event_reader: EventReader<InsertComponentEvents>,
    mut parent_query: Query<&mut FileSystemParent>,
    child_query: Query<&FileSystemChild>,
    entry_query: Query<&FileSystemEntry>,
    changelist_query: Query<&ChangelistEntry>,
    mut fs_state_query: Query<&mut FileSystemUiState>,
) {
    let project_root_entity = global.project_root_entity;
    let mut recent_parents: Option<HashMap<Entity, FileSystemParent>> = None;

    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for entry_entity in events.read::<FileSystemEntry>() {
            let entry = entry_query.get(entry_entity).unwrap();
            file_post_process::insert_ui_state_component(&mut commands, entry_entity, false);
            if *entry.kind == EntryKind::Directory {
                if recent_parents.is_none() {
                    recent_parents = Some(HashMap::new());
                }
                recent_parents
                    .as_mut()
                    .unwrap()
                    .insert(entry_entity, FileSystemParent::new());
            }
        }

        // on FileSystemRootChild Insert Event
        for child_entity in events.read::<FileSystemRootChild>() {
            // Add children to root parent
            let entry = entry_query.get(child_entity).unwrap();
            let mut parent = parent_query.get_mut(project_root_entity).unwrap();
            info!("Received FileSystemRootChild insert event");
            file_post_process::parent_add_child_entry(&mut parent, entry, child_entity);
        }

        // on FileSystemChild Insert Event
        for child_entity in events.read::<FileSystemChild>() {
            let entry = entry_query.get(child_entity).unwrap();

            // Get parent
            let Some(parent_entity) = child_query
                .get(child_entity)
                .unwrap()
                .parent_id
                .get(&client) else {
                panic!("FileSystemChild component of entry: `{}` has no parent component", *entry.name);
            };

            info!("Received FileSystemChild insert event");

            if let Ok(mut parent) = parent_query.get_mut(parent_entity) {
                file_post_process::parent_add_child_entry(&mut parent, entry, child_entity);
            } else {
                let Some(parent_map) = recent_parents.as_mut() else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", child_entity, parent_entity);
                };
                let Some(parent) = parent_map.get_mut(&parent_entity) else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", child_entity, parent_entity);
                };
                file_post_process::parent_add_child_entry(parent, entry, child_entity);
            }
        }
        // Add all parents now that the children were able to process
        // Note that we do it this way because Commands aren't flushed till the end of the system
        if let Some(parent_map) = recent_parents.as_mut() {
            for (entity, parent) in parent_map.drain() {
                commands.entity(entity).insert(parent);
            }
        }

        // on ChangelistEntry Insert Event
        for entity in events.read::<ChangelistEntry>() {
            commands.entity(entity).insert(ChangelistUiState::new());

            let entry = changelist_query.get(entity).unwrap();

            // associate status with file entry
            if let Some(file_entity) = entry.file_entity.get(&client) {
                let mut fs_state = fs_state_query.get_mut(file_entity).unwrap();
                fs_state.change_status = Some(*entry.status);
            }

            // insert into changelist resource
            global.changelist.insert(entry.file_entry_key(), entity);

            info!(
                "Received ChangelistEntry insert event. path: `{:?}`, name: `{:?}`",
                *entry.path, *entry.name
            );
        }
    }
}

pub fn update_component_events(
    mut event_reader: EventReader<UpdateComponentEvents>,
    entry_query: Query<&FileSystemEntry>,
) {
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

pub fn remove_component_events(
    client: Client,
    mut global: ResMut<Global>,
    mut parent_query: Query<&mut FileSystemParent>,
    mut event_reader: EventReader<RemoveComponentEvents>,
    mut fs_state_query: Query<&mut FileSystemUiState>,
) {
    for events in event_reader.iter() {
        for (_entity, _component) in events.read::<FileSystemEntry>() {
            info!("removed FileSystemEntry component from entity");
        }

        for (entity, _component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");

            let Ok(mut parent) = parent_query.get_mut(global.project_root_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }

        for (entity, component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");

            let Some(parent_entity) = component.parent_id.get(&client) else {
                continue;
            };
            let Ok(mut parent) = parent_query.get_mut(parent_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }
        for (_entity, component) in events.read::<ChangelistEntry>() {
            info!("removed ChangelistEntry component from entity");

            let entry = component.file_entry_key();
            global.changelist.remove(&entry);

            if let Some(file_entity) = component.file_entity.get(&client) {
                if let Ok(mut fs_state) = fs_state_query.get_mut(file_entity) {
                    fs_state.change_status = None;
                }
            }
        }
    }
}
