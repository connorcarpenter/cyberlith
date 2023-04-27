use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde};

pub struct FileSystemComponentsPlugin;

impl ProtocolPlugin for FileSystemComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<FileSystemEntry>()
            .add_component::<FileSystemChild>()
            .add_component::<FileSystemRootChild>();
    }
}

#[derive(Serde, PartialEq, Clone, Ord, PartialOrd, Eq)]
pub enum EntryKind {
    Directory,
    File,
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

// FileSystemChild
#[derive(Component, Replicate)]
pub struct FileSystemChild {
    pub parent_id: EntityProperty,
}

impl FileSystemChild {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// FileSystemChildRoot
#[derive(Component, Replicate)]
pub struct FileSystemRootChild;

impl FileSystemRootChild {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
