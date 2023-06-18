use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_log::info;
use naia_bevy_server::{
    events::{
        DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
        UpdateComponentEvents,
    },
    Server,
};

use vortex_proto::{
    components::{FileSystemEntry, HasParent, NoParent},
    resources::FileEntryKey,
};

use crate::resources::{
    fs_waitlist::{fs_process_insert, FSWaitlist, FSWaitlistInsert},
    GitManager, UserManager,
};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(_user_key, _entity) in event_reader.iter() {
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
        git_manager.workspace_mut(user.get_username()).delete_file(
            &mut commands,
            &mut server,
            entity,
        );
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
    fs_child_query: Query<&HasParent>,
    entry_key_query: Query<&FileEntryKey>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {
            info!("inserted FileSystemEntry");
            let entry = fs_entry_query.get(entity).unwrap();
            fs_process_insert(
                &mut commands,
                &mut server,
                FSWaitlistInsert::Entry(*entry.kind, (*entry.name).clone()),
                &user_manager,
                &mut git_manager,
                &mut fs_waiting_entities,
                &user_key,
                &entity,
            );
        }

        // on FileSystemRootChild Insert Event
        for (user_key, entity) in events.read::<NoParent>() {
            info!("inserted FileSystemRootChild");
            fs_process_insert(
                &mut commands,
                &mut server,
                FSWaitlistInsert::Parent(None),
                &user_manager,
                &mut git_manager,
                &mut fs_waiting_entities,
                &user_key,
                &entity,
            );
        }

        // on FileSystemChild Insert Event
        for (user_key, entity) in events.read::<HasParent>() {
            info!("inserted FileSystemChild");
            let entry = fs_child_query.get(entity).unwrap();
            let parent_entity = entry.parent_id.get(&server).unwrap();
            let parent_key = entry_key_query.get(parent_entity).unwrap();
            fs_process_insert(
                &mut commands,
                &mut server,
                FSWaitlistInsert::Parent(Some(parent_key.clone())),
                &user_manager,
                &mut git_manager,
                &mut fs_waiting_entities,
                &user_key,
                &entity,
            );
        }
    }
}

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (_user_key, _entity, _component) in events.read::<NoParent>() {
            info!("removed FileSystemRootChild component from entity");
            // TODO!
        }
        for (_user_key, _entity, _component) in events.read::<HasParent>() {
            info!("removed FileSystemChild component from entity");
            // TODO!
        }
    }
}

pub fn update_component_events(mut event_reader: EventReader<UpdateComponentEvents>) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_user_key, _entity) in events.read::<FileSystemEntry>() {
            // TODO!
        }
        // on FileSystemChild Update Event
        for (_user_key, _entity) in events.read::<HasParent>() {
            // TODO!
        }
    }
}
