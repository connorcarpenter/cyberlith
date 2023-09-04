
use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, ResMut},
};
use bevy_log::info;

use naia_bevy_client::{
    events::RemoveComponentEvents,
    Client,
};

use render_api::{
    base::CpuMesh,
    Assets,
};

use vortex_proto::components::{
    ChangelistEntry, Edge3d, Face3d, FileSystemChild, FileSystemEntry,
    FileSystemRootChild, Vertex3d,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    resources::{
        file_manager::FileManager,
        shape_manager::ShapeManager,
    },
};

pub fn remove_component_events(
    mut commands: Commands,
    client: Client,
    mut file_manager: ResMut<FileManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut event_reader: EventReader<RemoveComponentEvents>,
    mut parent_q: Query<&mut FileSystemParent>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    for events in event_reader.iter() {
        for (_entity, _component) in events.read::<FileSystemEntry>() {
            info!("removed FileSystemEntry component from entity");
        }

        for (entity, _component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");

            let Ok(mut parent) = parent_q.get_mut(file_manager.project_root_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }

        for (entity, component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");

            let Some(parent_entity) = component.parent_id.get(&client) else {
                continue;
            };
            let Ok(mut parent) = parent_q.get_mut(parent_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }
        for (_entity, component) in events.read::<ChangelistEntry>() {
            info!("removed ChangelistEntry component from entity");

            let entry = component.file_entry_key();
            file_manager.changelist.remove(&entry);

            if let Some(file_entity) = component.file_entity.get(&client) {
                if let Ok(mut fs_state) = fs_state_q.get_mut(file_entity) {
                    fs_state.change_status = None;
                }
            }
        }
        for (vertex_entity_3d, _) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, removed Vertex3d", vertex_entity_3d);

            shape_manager.cleanup_deleted_vertex(&mut commands, &vertex_entity_3d);
        }
        for (edge_3d_entity, _) in events.read::<Edge3d>() {
            info!("entity: `{:?}`, removed Edge3d", edge_3d_entity);

            shape_manager.cleanup_deleted_edge(&mut commands, &edge_3d_entity);
        }
        for (face_entity_3d, _) in events.read::<Face3d>() {
            info!("entity: `{:?}`, removed Face3d", face_entity_3d);

            shape_manager.cleanup_deleted_face_3d(&mut commands, &mut meshes, &face_entity_3d);
        }
    }
}
