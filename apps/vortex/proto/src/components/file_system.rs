use bevy_ecs::prelude::Component;
use naia_bevy_shared::{EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde};

use crate::resources::FileEntryKey;

pub struct FileSystemComponentsPlugin;

impl ProtocolPlugin for FileSystemComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<FileSystemEntry>()
            .add_component::<HasParent>()
            .add_component::<NoParent>()
            .add_component::<ChangelistEntry>();
    }
}

#[derive(Serde, PartialEq, Clone, Ord, PartialOrd, Eq, Copy, Hash, Debug)]
pub enum EntryKind {
    Directory,
    File,
}

#[derive(Serde, PartialEq, Clone, Ord, PartialOrd, Eq, Copy)]
pub enum ChangelistStatus {
    Modified,
    Created,
    Deleted,
}

// FileSystemEntry
#[derive(Component, Replicate)]
pub struct FileSystemEntry {
    pub name: Property<String>,
    pub kind: Property<EntryKind>,
}

impl FileSystemEntry {
    pub fn new(name: &str, kind: EntryKind) -> Self {
        Self::new_complete(name.to_string(), kind)
    }
}

// HasParent
#[derive(Component, Replicate)]
pub struct HasParent {
    pub parent_id: EntityProperty,
}

impl HasParent {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// NoParent
#[derive(Component, Replicate)]
pub struct NoParent;

impl NoParent {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// ChangelistEntry
#[derive(Component, Replicate)]
pub struct ChangelistEntry {
    pub kind: Property<EntryKind>,
    pub name: Property<String>,
    pub path: Property<String>,
    pub status: Property<ChangelistStatus>,
    pub file_entity: EntityProperty,
}

impl ChangelistEntry {
    pub fn new(kind: EntryKind, name: &str, path: &str, status: ChangelistStatus) -> Self {
        Self::new_complete(kind, name.to_string(), path.to_string(), status)
    }

    pub fn file_entry_key(&self) -> FileEntryKey {
        FileEntryKey::new(&self.path, &self.name, *self.kind)
    }
}
