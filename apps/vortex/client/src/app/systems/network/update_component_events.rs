use bevy_ecs::{
    event::EventReader,
    system::{Query, ResMut},
};
use bevy_log::info;

use naia_bevy_client::{events::UpdateComponentEvents, Client};

use vortex_proto::components::{AnimFrame, ChangelistEntry, EdgeAngle, FileSystemChild, FileSystemEntry, FileSystemRootChild, ShapeName, Vertex3d};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemEntryLocal},
    resources::{
        canvas::Canvas,
        file_manager::{get_full_path, FileManager},
    },
};

pub fn update_component_events(
    client: Client,
    mut event_reader: EventReader<UpdateComponentEvents>,
    file_manager: ResMut<FileManager>,
    mut canvas: ResMut<Canvas>,
    entry_q: Query<(&FileSystemEntry, Option<&FileSystemChild>)>,
    mut entry_local_q: Query<&mut FileSystemEntryLocal>,
    mut cl_q: Query<(&ChangelistEntry, &mut ChangelistUiState)>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_, entry_entity) in events.read::<FileSystemEntry>() {
            let (entry, _) = entry_q.get(entry_entity).unwrap();
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
                    info!(
                        "change path for child entity: `{:?}`. path was: `{:?}`, now is `{:?}`",
                        cl_child_entity, old_path, new_path
                    );

                    let (_, mut cl_child_state) = cl_q.get_mut(*cl_child_entity).unwrap();
                    cl_child_state.display_path = new_path;
                }
            }
            let mut entry_local = entry_local_q.get_mut(entry_entity).unwrap();
            entry_local.name = entry_name;
        }
        // on FileSystemRootChild Update Event
        for (_, child_entity) in events.read::<FileSystemRootChild>() {
            let (entry, _) = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemRootChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on FileSystemChild Update Event
        for (_, child_entity) in events.read::<FileSystemChild>() {
            let (entry, _) = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on Shape Update Event
        let mut updated_shapes = false;
        for (_, _) in events.read::<Vertex3d>() {
            updated_shapes = true;
            break;
        }
        for (_, _) in events.read::<EdgeAngle>() {
            updated_shapes = true;
            break;
        }
        if updated_shapes {
            canvas.queue_resync_shapes();
        }
        for (_tick, _entity) in events.read::<ShapeName>() {
            todo!();
        }
        for (_tick, _entity) in events.read::<AnimFrame>() {
            todo!();
        }
    }
}
