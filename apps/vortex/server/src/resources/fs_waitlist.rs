use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use bevy_log::info;
use naia_bevy_server::{Server, UserKey};

use vortex_proto::{components::EntryKind, resources::FileEntryKey};

use crate::resources::{GitManager, TabManager, UserManager, VertexManager};

pub enum FSWaitlistInsert {
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

pub fn fs_process_insert(
    commands: &mut Commands,
    server: &mut Server,
    insert: FSWaitlistInsert,
    user_manager: &UserManager,
    git_manager: &mut GitManager,
    tab_manager: &mut TabManager,
    vertex_manager: &mut VertexManager,
    fs_waiting_entities: &mut HashMap<Entity, FSWaitlist>,
    user_key: &UserKey,
    entity: &Entity,
    key_query: &Query<&FileEntryKey>,
) {
    if !fs_waiting_entities.contains_key(&entity) {
        fs_waiting_entities.insert(*entity, FSWaitlist::new());
    }
    let waitlist = fs_waiting_entities.get_mut(&entity).unwrap();

    match insert {
        FSWaitlistInsert::Entry(kind, name) => {
            waitlist.set_entry(kind, name);
        }
        FSWaitlistInsert::Parent(parent) => {
            waitlist.set_parent(parent);
        }
    }

    if waitlist.is_ready() {
        info!("New Entity is ready to be spawned!");
        let insert = fs_waiting_entities.remove(entity).unwrap();
        fs_process_insert_complete(
            commands,
            server,
            user_manager,
            git_manager,
            user_key,
            entity,
            insert,
        );

        if let Some(tab_id) = tab_manager.remove_waiting_open(user_key, entity) {
            tab_manager.open_tab(
                commands,
                server,
                user_manager,
                git_manager,
                vertex_manager,
                key_query,
                user_key,
                &tab_id,
                entity,
            );
        }
    }
}

fn fs_process_insert_complete(
    commands: &mut Commands,
    server: &mut Server,
    user_manager: &UserManager,
    git_manager: &mut GitManager,
    user_key: &UserKey,
    file_entity: &Entity,
    entry: FSWaitlist,
) {
    let Some(user) = user_manager.user_info(user_key) else {
        panic!("user not found!");
    };
    let (name, kind, parent) = entry.decompose();
    info!("creating file: {}", name);
    let key = FileEntryKey::new_with_parent(parent.clone(), &name, kind);
    git_manager
        .workspace_mut(user.get_username())
        .on_client_create_file(commands, server, &name, *file_entity, parent, &key);

    commands.entity(*file_entity).insert(key);
}
