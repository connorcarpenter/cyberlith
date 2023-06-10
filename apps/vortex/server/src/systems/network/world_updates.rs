use std::collections::HashMap;

use bevy_ecs::{entity::Entity, event::EventReader, system::{Commands, Local, Query, Res, ResMut}};
use bevy_log::info;

use naia_bevy_server::{events::{
    DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
    UpdateComponentEvents,
}, Server, CommandsExt};

use vortex_proto::{components::{FileSystemChild, FileSystemEntry, FileSystemRootChild}, resources::FileEntryKey};

use crate::resources::{GitManager, UserManager, fs_waitlist::{FSWaitlist, fs_process_insert, FSWaitlistInsert}};

pub fn spawn_entity_events(
    mut event_reader: EventReader<SpawnEntityEvent>
) {
    for SpawnEntityEvent(user_key, entity) in event_reader.iter() {
        info!("spawned entity");
    }
}

pub fn despawn_entity_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut event_reader: EventReader<DespawnEntityEvent>,
) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        info!("despawned entity");

        let Some(user) = user_manager.user_info(user_key) else {
            panic!("user not found");
        };
        let entities_to_despawn = git_manager.workspace_mut(user.get_username()).delete_file(&mut commands, &mut server, entity);

        for child_entity in entities_to_despawn {
            commands.entity(child_entity).take_authority(&mut server).despawn();
            info!("child entity has been despawned");
        }
    }
}

pub fn insert_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut fs_waiting_entities: Local<HashMap<Entity, FSWaitlist>>,
    mut event_reader: EventReader<InsertComponentEvents>,
    fs_entry_query: Query<&FileSystemEntry>,
    fs_child_query: Query<&FileSystemChild>,
    entry_key_query: Query<&FileEntryKey>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {
            info!("inserted FileSystemEntry");
            let entry = fs_entry_query.get(entity).unwrap();
            fs_process_insert(&mut commands, &mut server, FSWaitlistInsert::Entry(*entry.kind, (*entry.name).clone()), &user_manager, &mut git_manager, &mut fs_waiting_entities, &user_key, &entity);
        }

        // on FileSystemRootChild Insert Event
        for (user_key, entity) in events.read::<FileSystemRootChild>() {
            info!("inserted FileSystemRootChild");
            fs_process_insert(&mut commands, &mut server, FSWaitlistInsert::Parent(None), &user_manager, &mut git_manager, &mut fs_waiting_entities, &user_key, &entity);
        }

        // on FileSystemChild Insert Event
        for (user_key, entity) in events.read::<FileSystemChild>() {
            info!("inserted FileSystemChild");
            let entry = fs_child_query.get(entity).unwrap();
            let parent_entity = entry.parent_id.get(&server).unwrap();
            let parent_key = entry_key_query.get(parent_entity).unwrap();
            fs_process_insert(&mut commands, &mut server, FSWaitlistInsert::Parent(Some(parent_key.clone())), &user_manager, &mut git_manager, &mut fs_waiting_entities, &user_key, &entity);
        }
    }
}

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (user_key, entity, component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");
            // TODO!
        }
        for (user_key, entity, component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");
            // TODO!
        }
    }
}

pub fn update_component_events(mut event_reader: EventReader<UpdateComponentEvents>) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {
            // TODO!
        }
        // on FileSystemChild Update Event
        for (user_key, entity) in events.read::<FileSystemChild>() {
            // TODO!
        }
    }
}
