use std::collections::BTreeMap;

use bevy_ecs::prelude::{Component, Entity};

use vortex_proto::components::EntryKind;

use crate::app::ui::ModalRequestHandle;

// FileSystemParent
#[derive(Component)]
pub struct FileSystemParent {
    children_ids: BTreeMap<(EntryKind, String), Entity>,
}

impl FileSystemParent {
    pub fn new() -> Self {
        Self {
            children_ids: BTreeMap::new(),
        }
    }

    pub fn add_child(&mut self, kind: EntryKind, name: String, child_id: Entity) {
        self.children_ids
            .insert((kind, name.to_lowercase()), child_id);
    }

    // pub fn remove_child(&mut self, child_id: Entity) {
    //     self.children_ids.retain(|&id| id != child_id);
    // }
    //
    // pub fn has_children(&self) -> bool {
    //     !self.children_ids.is_empty()
    // }

    pub fn get_children(&self) -> Vec<Entity> {
        let mut output = Vec::new();
        for (_, v) in self.children_ids.iter() {
            output.push(*v);
        }
        output
    }
}

// FileSystemUiState
#[derive(Component)]
pub struct FileSystemUiState {
    pub selected: bool,
    pub opened: bool,
    pub context_menu_response: Option<ContextMenuAction>,
    pub modal_request: Option<(ModalRequestType, ModalRequestHandle)>,
}

#[derive(Clone)]
pub enum ContextMenuAction {
    None,
    NewFile,
    NewDirectory,
    Rename,
    Delete,
    Cut,
    Copy,
    Paste,
}

#[derive(Clone)]
pub enum ModalRequestType {
    NewFile(Entity),
    Delete(Entity),
    Rename,
}

impl FileSystemUiState {
    pub fn new() -> Self {
        Self {
            selected: false,
            opened: false,
            context_menu_response: None,
            modal_request: None,
        }
    }

    pub fn new_root() -> Self {
        let mut new = Self::new();
        new.opened = true;
        new
    }
}
