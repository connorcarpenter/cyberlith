use std::collections::HashSet;

use bevy_ecs::entity::Entity;

use vortex_proto::{components::ChangelistStatus, resources::FileEntryKey};

#[derive(Clone)]
pub struct FileEntryValue {
    entity: Entity,
    parent: Option<FileEntryKey>,
    children: Option<HashSet<FileEntryKey>>,
}

impl FileEntryValue {
    pub fn new(
        entity: Entity,
        parent: Option<FileEntryKey>,
        children: Option<HashSet<FileEntryKey>>,
    ) -> Self {
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

    pub fn children(&self) -> Option<&HashSet<FileEntryKey>> {
        self.children.as_ref()
    }

    pub fn add_child(&mut self, key: FileEntryKey) {
        self.children
            .get_or_insert_with(|| HashSet::new())
            .insert(key);
    }

    pub fn remove_child(&mut self, key: &FileEntryKey) {
        if let Some(children) = self.children.as_mut() {
            children.remove(&key);
        }
    }
}

#[derive(Clone)]
pub struct ChangelistValue {
    entity: Entity,
    status: ChangelistStatus,
}

impl ChangelistValue {
    pub fn new(entity: Entity, status: ChangelistStatus) -> Self {
        Self { entity, status }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn status(&self) -> ChangelistStatus {
        self.status
    }
}
