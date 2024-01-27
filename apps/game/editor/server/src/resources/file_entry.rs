use std::collections::HashSet;

use bevy_ecs::entity::Entity;

use editor_proto::{components::FileExtension, resources::FileKey};

#[derive(Clone)]
pub struct FileEntryValue {
    entity: Entity,
    extension: Option<FileExtension>,
    parent: Option<FileKey>,
    children: Option<HashSet<FileKey>>,
    dependencies: Option<HashSet<FileKey>>,
}

impl FileEntryValue {
    pub fn new(
        entity: Entity,
        extension: Option<FileExtension>,
        parent: Option<FileKey>,
        children: Option<HashSet<FileKey>>,
    ) -> Self {
        Self {
            entity,
            parent,
            children,
            extension,
            dependencies: None,
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = entity;
    }

    pub fn extension(&self) -> Option<FileExtension> {
        self.extension
    }

    pub fn parent(&self) -> Option<&FileKey> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> Option<&HashSet<FileKey>> {
        self.children.as_ref()
    }

    pub fn add_child(&mut self, key: FileKey) {
        self.children
            .get_or_insert_with(|| HashSet::new())
            .insert(key);
    }

    pub fn remove_child(&mut self, key: &FileKey) {
        if let Some(children) = self.children.as_mut() {
            children.remove(&key);
        }
    }

    pub fn dependencies(&self) -> Option<&HashSet<FileKey>> {
        self.dependencies.as_ref()
    }

    pub fn add_dependency(&mut self, key: &FileKey) {
        if self.dependencies.is_none() {
            self.dependencies = Some(HashSet::new());
        }
        let dependencies = self.dependencies.as_mut().unwrap();
        if dependencies.contains(key) {
            panic!("dependency already exists");
        }
        dependencies.insert(key.clone());
    }

    pub fn remove_dependency(&mut self, key: &FileKey) {
        let dependencies = self.dependencies.as_mut().unwrap();
        if !dependencies.remove(&key) {
            panic!("dependency not found");
        }
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

    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    pub fn set_content(&mut self, content: Box<[u8]>) {
        self.content = Some(content.into());
    }

    pub fn get_content(&self) -> Option<&[u8]> {
        self.content.as_ref().map(|c| c.as_ref())
    }

    pub fn delete_content(&mut self) {
        self.content = None;
    }
}
