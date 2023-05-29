use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use vortex_proto::components::{EntryKind, FileSystemEntry};

use crate::app::components::file_system::{FileSystemParent, FileSystemUiState};

pub fn on_added_entry(
    commands: &mut Commands,
    entry: &FileSystemEntry,
    entry_entity: Entity,
    ui_should_select: bool,
) {
    // Add FileSystemUiState to all entities
    let mut ui_state = FileSystemUiState::new();
    if ui_should_select {
        ui_state.selected = true;
    }
    commands.entity(entry_entity).insert(ui_state);
}

pub fn on_added_child(
    parent: &mut FileSystemParent,
    child_entry: &FileSystemEntry,
    child_entity: Entity,
) {
    let entry_kind = (*(child_entry.kind)).clone();
    let child_name = (*(child_entry.name)).clone();
    parent.add_child(entry_kind, child_name, child_entity);
}
