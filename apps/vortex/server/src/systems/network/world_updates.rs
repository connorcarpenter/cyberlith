use std::collections::HashMap;

use bevy_ecs::{entity::Entity, event::EventReader, system::{Commands, Local, Query, Res, ResMut}};
use bevy_log::{info, warn};

use naia_bevy_server::{events::{
    DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
    UpdateComponentEvents,
}, Server, UserKey, CommandsExt};

use vortex_proto::{components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild}, resources::FileEntryKey};

use crate::resources::{GitManager, UserManager};

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
        let entities_to_despawn = git_manager.workspace_mut(user.get_username()).delete_file(entity);

        for child_entity in entities_to_despawn {
            commands.entity(child_entity).take_authority(&mut server).despawn();
            info!("child entity has been despawned");
        }
    }
}

enum FSWaitlistInsert{
    Entry(EntryKind, String),
    Parent(Option<FileEntryKey>),
}

pub struct FSWaitlist {
    entry: Option<(EntryKind, String)>,
    parent: Option<Option<FileEntryKey>>,
}

impl FSWaitlist {
    fn new() -> Self {
        Self {
            entry: None,
            parent: None,
        }
    }

    fn is_ready(&self) -> bool {
        self.entry.is_some() && self.parent.is_some()
    }

    fn set_parent(&mut self, key: Option<FileEntryKey>) {
        self.parent = Some(key);
    }

    fn set_entry(&mut self, kind: EntryKind, name: String) {
        self.entry = Some((kind, name));
    }

    pub(crate) fn decompose(self) -> (String, EntryKind, Option<FileEntryKey>) {
        let (kind, name) = self.entry.unwrap();
        let parent = self.parent.unwrap();
        (name, kind, parent)
    }
}

pub fn insert_component_events(
    mut commands: Commands,
    server: Server,
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
            fs_process_insert(&mut commands, FSWaitlistInsert::Entry(*entry.kind, (*entry.name).clone()), &user_manager, &mut git_manager, &mut fs_waiting_entities, &user_key, &entity);
        }

        // on FileSystemRootChild Insert Event
        for (user_key, entity) in events.read::<FileSystemRootChild>() {
            info!("inserted FileSystemRootChild");
            fs_process_insert(&mut commands, FSWaitlistInsert::Parent(None), &user_manager, &mut git_manager, &mut fs_waiting_entities, &user_key, &entity);
        }

        // on FileSystemChild Insert Event
        for (user_key, entity) in events.read::<FileSystemChild>() {
            info!("inserted FileSystemChild");
            let entry = fs_child_query.get(entity).unwrap();
            let parent_entity = entry.parent_id.get(&server).unwrap();
            let parent_key = entry_key_query.get(parent_entity).unwrap();
            fs_process_insert(&mut commands, FSWaitlistInsert::Parent(Some(parent_key.clone())), &user_manager, &mut git_manager, &mut fs_waiting_entities, &user_key, &entity);
        }
    }
}

fn fs_process_insert(
    commands: &mut Commands,
    insert: FSWaitlistInsert,
    user_manager: &UserManager,
    git_manager: &mut GitManager,
    fs_waiting_entities: &mut HashMap<Entity, FSWaitlist>,
    user_key: &UserKey,
    entity: &Entity
) {
    if !fs_waiting_entities.contains_key(&entity) {
        fs_waiting_entities.insert(*entity, FSWaitlist::new());
    }
    let waitlist = fs_waiting_entities.get_mut(&entity).unwrap();

    match insert {
        FSWaitlistInsert::Entry(kind, name) => {
            waitlist.set_entry(kind, name);
        },
        FSWaitlistInsert::Parent(parent) => {
            waitlist.set_parent(parent);
        },
    }

    if waitlist.is_ready() {
        info!("New Entity is ready to be spawned!");
        let insert = fs_waiting_entities.remove(entity).unwrap();
        fs_process_insert_complete(commands, user_manager, git_manager, user_key, entity, insert);
    }
}

fn fs_process_insert_complete(
    commands: &mut Commands,
    user_manager: &UserManager,
    git_manager: &mut GitManager,
    user_key: &UserKey,
    entity: &Entity,
    entry: FSWaitlist
) {
    let Some(user) = user_manager.user_info(user_key) else {
        panic!("user not found!");
    };
    let (name, kind, parent) = entry.decompose();
    git_manager.workspace_mut(user.get_username()).create_file(&name, kind, *entity, parent.clone());

    commands.entity(*entity).insert(FileEntryKey::new_with_parent(parent, &name, kind));
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
