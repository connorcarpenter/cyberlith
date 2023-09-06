use bevy_ecs::{
    event::EventReader,
    system::{Query, ResMut},
};
use bevy_log::info;

use naia_bevy_client::events::UpdateComponentEvents;

use vortex_proto::components::{FileSystemChild, FileSystemEntry, FileSystemRootChild, Vertex3d};

use crate::app::{components::file_system::ChangelistUiState, resources::{file_manager::FileManager, shape_manager::ShapeManager}};

pub fn update_component_events(
    mut event_reader: EventReader<UpdateComponentEvents>,
    file_manager: ResMut<FileManager>,
    mut shape_manager: ResMut<ShapeManager>,
    entry_q: Query<&FileSystemEntry>,
    mut cl_q: Query<&mut ChangelistUiState>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_, entry_entity) in events.read::<FileSystemEntry>() {
            let entry = entry_q.get(entry_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemEntry: `{:?}` ({:?})",
                entry_entity, entry_name
            );
            if let Some(cl_entity) = file_manager.get_file_changelist_entity(&entry_entity) {
                let mut cl_state = cl_q.get_mut(cl_entity).unwrap();
                cl_state.display_name = entry_name;
            }
        }
        // on FileSystemRootChild Update Event
        for (_, child_entity) in events.read::<FileSystemRootChild>() {
            let entry = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemRootChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on FileSystemChild Update Event
        for (_, child_entity) in events.read::<FileSystemChild>() {
            let entry = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on Vertex3d Update Event
        let mut updated_vertices = false;
        for (_, _) in events.read::<Vertex3d>() {
            updated_vertices = true;
            break;
        }
        if updated_vertices {
            shape_manager.recalculate_shapes();
        }
    }
}
