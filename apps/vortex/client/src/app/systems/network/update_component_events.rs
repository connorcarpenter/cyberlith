use bevy_ecs::{
    event::EventReader,
    system::{Query, ResMut},
    entity::Entity,
};
use bevy_log::info;

use naia_bevy_client::{events::UpdateComponentEvents, Client};

use vortex_proto::components::{ChangelistEntry, FileSystemChild, FileSystemEntry, FileSystemRootChild, Vertex3d};

use crate::app::{components::file_system::{ChangelistUiState, FileSystemEntryLocal}, resources::{file_manager::FileManager, shape_manager::ShapeManager}};

pub fn update_component_events(
    client: Client,
    mut event_reader: EventReader<UpdateComponentEvents>,
    file_manager: ResMut<FileManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut entry_q: Query<(&FileSystemEntry, &mut FileSystemEntryLocal, Option<&FileSystemChild>)>,
    mut cl_q: Query<(&ChangelistEntry, &mut ChangelistUiState)>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_, entry_entity) in events.read::<FileSystemEntry>() {
            let (entry, _, _) = entry_q.get(entry_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemEntry: `{:?}` ({:?})",
                entry_entity, entry_name
            );
            if let Some(cl_entity) = file_manager.get_file_changelist_entity(&entry_entity) {
                let (_, mut cl_state) = cl_q.get_mut(cl_entity).unwrap();
                cl_state.display_name = entry_name.clone();
            }
            if let Some(cl_children) = file_manager.get_file_changelist_children(&entry_entity) {
                for cl_child_entity in cl_children.iter() {
                    let (cl_entry, old_child_state) = cl_q.get(*cl_child_entity).unwrap();
                    let child_file_entity = cl_entry.file_entity.get(&client).unwrap();

                    let old_path = old_child_state.display_path.clone();
                    let new_path = get_full_path(&client, &entry_q, child_file_entity);
                    info!("change path for child entity: `{:?}`. path was: `{:?}`, now is `{:?}`", cl_child_entity, old_path, new_path);

                    let (_, mut cl_child_state) = cl_q.get_mut(*cl_child_entity).unwrap();
                    cl_child_state.display_path = new_path;
                }
            }
            let (_, mut entry_local, _) = entry_q.get_mut(entry_entity).unwrap();
            entry_local.name = entry_name;
        }
        // on FileSystemRootChild Update Event
        for (_, child_entity) in events.read::<FileSystemRootChild>() {
            let (entry, _, _) = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemRootChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on FileSystemChild Update Event
        for (_, child_entity) in events.read::<FileSystemChild>() {
            let (entry, _, _) = entry_q.get(child_entity).unwrap();
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

fn get_full_path(
    client: &Client,
    fs_q: &Query<(&FileSystemEntry, &mut FileSystemEntryLocal, Option<&FileSystemChild>)>,
    file_entity: Entity
) -> String {
    let mut path = String::new();

    let (_, _, parent) = fs_q.get(file_entity).unwrap();
    if let Some(parent_entity) = parent {
        let mut current_entity = parent_entity.parent_id.get(client).unwrap();

        loop {
            let (entry, _, parent) = fs_q.get(current_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            path.insert_str(0, &entry_name);
            if let Some(parent_entity) = parent {
                current_entity = parent_entity.parent_id.get(client).unwrap();
                path.insert_str(0, "/");
            } else {
                break;
            }
        }
    }

    path
}
