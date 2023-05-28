use std::collections::HashMap;

use bevy_ecs::{system::Commands, entity::Entity};

use vortex_proto::components::{EntryKind, FileSystemEntry};

use crate::app::components::file_system::{FileSystemParent, FileSystemUiState};

pub fn on_added_entry(commands: &mut Commands, entry: &FileSystemEntry, entry_entity: Entity, recent_parents: &mut Option<HashMap<Entity, FileSystemParent>>) {

    // Add FileSystemParent to directories
    if *entry.kind == EntryKind::Directory {
        if recent_parents.is_none() {
            *recent_parents = Some(HashMap::new());
        }
        let map = recent_parents.as_mut().unwrap();
        map.insert(entry_entity, FileSystemParent::new());
    }
    // Add FileSystemUiState to all entities
    commands
        .entity(entry_entity)
        .insert(FileSystemUiState::new());
}

pub fn on_added_child(parent: &mut FileSystemParent, child_entry: &FileSystemEntry, child_entity: Entity) {
    let entry_kind = (*(child_entry.kind)).clone();
    let child_name = (*(child_entry.name)).clone();
    parent.add_child(entry_kind, child_name, child_entity);
}