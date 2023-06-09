use bevy_ecs::{entity::Entity, component::Component};

use crate::components::{ChangelistStatus, EntryKind};

#[derive(Clone, Eq, Hash, PartialEq, Component)]
pub struct FileEntryKey {
    name: String,
    path: String,
    kind: EntryKind,
}

impl FileEntryKey {
    pub fn new(path: &str, name: &str, kind: EntryKind) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            kind,
        }
    }

    pub fn new_with_parent(parent: Option<FileEntryKey>, name: &str, kind: EntryKind) -> Self {
        let path = match &parent {
            Some(parent_key) => {
                parent_key.path_for_children()
            },
            None => "".to_string(),
        };

        Self::new(&path, name, kind)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    pub fn path_for_children(&self) -> String {
        format!("{}{}/", self.path, self.name)
    }
}

#[derive(Clone)]
pub struct FileEntryValue {
    entity: Entity,
    parent: Option<FileEntryKey>,
    children: Option<Vec<FileEntryKey>>,
}

impl FileEntryValue {
    pub fn new(entity: Entity, parent: Option<FileEntryKey>, children: Option<Vec<FileEntryKey>>) -> Self {
        Self {
            entity,
            parent,
            children,
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn parent(&self) -> Option<&FileEntryKey> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> Option<&Vec<FileEntryKey>> {
        self.children.as_ref()
    }

    pub fn add_child(&mut self, key: FileEntryKey) {
        self.children.get_or_insert_with(|| Vec::new()).push(key);
    }
}

#[derive(Clone)]
pub struct ChangelistValue {
    entity: Entity,
    status: ChangelistStatus,
}

impl ChangelistValue {
    pub fn new(entity: Entity, status: ChangelistStatus) -> Self {
        Self {
            entity,
            status
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn status(&self) -> ChangelistStatus {
        self.status
    }
}