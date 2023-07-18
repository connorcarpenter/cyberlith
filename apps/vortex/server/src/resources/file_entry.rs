use std::collections::HashSet;

use bevy_ecs::entity::Entity;

use vortex_proto::{resources::FileEntryKey, FileExtension};

#[derive(Clone)]
pub struct FileEntryValue {
    entity: Entity,
    parent: Option<FileEntryKey>,
    children: Option<HashSet<FileEntryKey>>,
    extension: Option<FileExtension>,
}

impl FileEntryValue {
    pub fn new(
        entity: Entity,
        parent: Option<FileEntryKey>,
        children: Option<HashSet<FileEntryKey>>,
        extension: Option<FileExtension>,
    ) -> Self {
        Self {
            entity,
            parent,
            children,
            extension,
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = entity;
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

    pub fn extension(&self) -> Option<FileExtension> {
        self.extension
    }
}

#[derive(Clone)]
pub struct ChangelistValue {
    entity: Entity,
    content: Option<Box<[u8]>>,
}

impl ChangelistValue {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            content: None,
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn set_content(&mut self, content: Box<[u8]>) {
        self.content = Some(content.into());
    }

    pub fn get_content(&self) -> Option<&[u8]> {
        self.content.as_ref().map(|c| c.as_ref())
    }
}
