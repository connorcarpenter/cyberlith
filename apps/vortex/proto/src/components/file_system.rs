use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde};

pub struct FileSystemComponentsPlugin;

impl ProtocolPlugin for FileSystemComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<FileSystemEntry>()
            .add_component::<FileSystemParent>()
            .add_component::<FileSystemRoot>();
    }
}

#[derive(Serde, PartialEq, Clone)]
pub enum EntryKind {
    File,
    Directory
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

// FileSystemParent
#[derive(Component, Replicate)]
pub struct FileSystemParent {
    pub id: EntityProperty,
}

// FileSystemRoot
#[derive(Component, Replicate)]
pub struct FileSystemRoot;