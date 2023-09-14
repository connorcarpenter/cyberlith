use bevy_ecs::component::Component;

use crate::components::EntryKind;

#[derive(Clone, Eq, Hash, PartialEq, Component, Ord, PartialOrd, Debug)]
pub struct FileEntryKey {
    kind: EntryKind,
    name: String,
    path: String,
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
            Some(parent_key) => parent_key.full_path(),
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

    pub fn full_path(&self) -> String {
        format!("{}{}", self.path, self.name)
    }
}
