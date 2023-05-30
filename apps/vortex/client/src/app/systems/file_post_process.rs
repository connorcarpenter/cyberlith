
use bevy_ecs::{entity::Entity, system::Commands};
use bevy_log::info;

use vortex_proto::components::FileSystemEntry;

use crate::app::components::file_system::{FileSystemParent, FileSystemUiState};

pub fn insert_ui_state_component(
    commands: &mut Commands,
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

pub fn parent_add_child_entry(
    parent: &mut FileSystemParent,
    child_entry: &FileSystemEntry,
    child_entity: Entity,
) {
    let entry_kind = (*(child_entry.kind)).clone();
    let child_name = (*(child_entry.name)).clone();

    info!("added child of name: `{}`, to parent", &child_name);

    parent.add_child(entry_kind, child_name, child_entity);
}
