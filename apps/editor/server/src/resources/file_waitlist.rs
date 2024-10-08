use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use naia_bevy_server::{Server, UserKey};

use editor_proto::{components::EntryKind, resources::FileKey};

use crate::resources::{GitManager, TabManager, UserManager};

pub enum FSWaitlistInsert {
    Entry(EntryKind, String),
    Parent(Option<FileKey>),
}

pub struct FSWaitlist {
    entry: Option<(EntryKind, String)>,
    parent: Option<Option<FileKey>>,
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

    fn set_parent(&mut self, key: Option<FileKey>) {
        self.parent = Some(key);
    }

    fn set_entry(&mut self, kind: EntryKind, name: String) {
        self.entry = Some((kind, name));
    }

    pub(crate) fn decompose(self) -> (String, EntryKind, Option<FileKey>) {
        let (kind, name) = self.entry.unwrap();
        let parent = self.parent.unwrap();
        (name, kind, parent)
    }
}

pub fn file_process_insert(
    commands: &mut Commands,
    server: &mut Server,
    insert: FSWaitlistInsert,
    user_manager: &UserManager,
    git_manager: &mut GitManager,
    tab_manager: &mut TabManager,
    fs_waiting_entities: &mut HashMap<Entity, FSWaitlist>,
    user_key: &UserKey,
    entity: &Entity,
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

        tab_manager.complete_waiting_open(user_key, entity);
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
    let Some(user_session_data) = user_manager.user_session_data(user_key) else {
        panic!("user not found!");
    };

    let project_key = user_session_data.project_key().unwrap();

    let (name, kind, parent) = entry.decompose();

    let key = FileKey::new_with_parent(parent.clone(), &name, kind);
    git_manager.on_client_create_file(
        commands,
        server,
        &project_key,
        &name,
        *file_entity,
        parent,
        &key,
    );
}
