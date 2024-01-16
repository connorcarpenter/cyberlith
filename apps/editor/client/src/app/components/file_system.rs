use std::collections::BTreeMap;

use bevy_ecs::prelude::{Component, Entity};

use editor_proto::{
    components::{ChangelistStatus, EntryKind},
    messages::ChangelistAction,
};

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

    pub fn has_child(&self, kind: EntryKind, name: &str) -> bool {
        self.children_ids.contains_key(&(kind, name.to_lowercase()))
    }

    pub fn add_child(&mut self, kind: EntryKind, name: String, child_id: Entity) {
        self.children_ids
            .insert((kind, name.to_lowercase()), child_id);
    }

    pub fn remove_child(&mut self, child_id: &Entity) {
        self.children_ids.retain(|_, id| id != child_id);
    }
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

// FileSystemEntryLocal
#[derive(Component)]
pub struct FileSystemEntryLocal {
    pub name: String,
}

impl FileSystemEntryLocal {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

// FileSystemUiState
#[derive(Component)]
pub struct FileSystemUiState {
    pub selected: bool,
    pub opened: bool,
    pub context_menu_response: Option<ContextMenuAction>,
    pub modal_request: Option<(ModalRequestType, ModalRequestHandle)>,
    pub change_status: Option<ChangelistStatus>,
}

impl FileSystemUiState {
    pub fn new() -> Self {
        Self {
            selected: false,
            opened: false,
            context_menu_response: None,
            modal_request: None,
            change_status: None,
        }
    }

    pub fn new_root() -> Self {
        let mut new = Self::new();
        new.opened = true;
        new
    }
}

// ChangelistUiState
#[derive(Component)]
pub struct ChangelistUiState {
    pub selected: bool,
    // the first Option is for whether there is a response
    // the second Option is whether there is an Action or a No-Op
    pub context_menu_response: Option<Option<ChangelistAction>>,
    // this name includes renames, shouldn't be used for a key!
    pub display_name: String,
    pub display_path: String,
}

impl ChangelistUiState {
    pub fn new(display_name: &str, display_path: &str) -> Self {
        Self {
            selected: false,
            context_menu_response: None,
            display_name: display_name.to_string(),
            display_path: display_path.to_string(),
        }
    }
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
    // NewFileRequest has an optional parent directory entity, or None if the new file should be at root
    NewFile(Option<Entity>),
    // NewFileRequest has an optional parent directory entity, or None if the new file should be at root
    NewDirectory(Option<Entity>),
    // DeleteFileRequest has the file entity to delete
    Delete(Entity),
    Rename,
}
