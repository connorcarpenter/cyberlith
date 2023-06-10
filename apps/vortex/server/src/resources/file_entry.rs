use bevy_ecs::entity::Entity;

use vortex_proto::{components::ChangelistStatus, resources::FileEntryKey};

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