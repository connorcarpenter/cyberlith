use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, ResMut},
};
use bevy_log::info;

use naia_bevy_client::{events::RemoveComponentEvents, Client};

use render_api::{base::CpuMesh, Assets};

use vortex_proto::components::{
    ChangelistEntry, ChangelistStatus, Edge3d, Face3d, FileDependency, FileSystemChild,
    FileSystemEntry, FileSystemRootChild, Vertex3d,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    resources::{
        canvas::Canvas, edge_manager::EdgeManager, face_manager::FaceManager,
        file_manager::FileManager, input_manager::InputManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn remove_component_events(
    mut commands: Commands,
    mut client: Client,
    mut canvas: ResMut<Canvas>,
    mut file_manager: ResMut<FileManager>,
    mut input_manager: ResMut<InputManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut tab_manager: ResMut<TabManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut event_reader: EventReader<RemoveComponentEvents>,
    mut parent_q: Query<&mut FileSystemParent>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    for events in event_reader.iter() {
        for (entity, _component) in events.read::<FileSystemEntry>() {
            info!("entity: `{:?}`, removed FileSystemEntry", entity);

            file_manager.on_file_delete(&mut client, &mut tab_manager, &entity);
        }

        for (entity, _component) in events.read::<FileSystemRootChild>() {
            info!("entity: `{:?}`, removed FileSystemRootChild", entity);

            let Ok(mut parent) = parent_q.get_mut(file_manager.project_root_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }

        for (entity, component) in events.read::<FileSystemChild>() {
            info!("entity: `{:?}`, removed FileSystemChild", entity);

            let Some(parent_entity) = component.parent_id.get(&client) else {
                continue;
            };
            let Ok(mut parent) = parent_q.get_mut(parent_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }
        for (entity, component) in events.read::<ChangelistEntry>() {
            info!("entity: `{:?}`, removed ChangelistEntry", entity);

            let entry = component.file_key();
            file_manager.remove_changelist_entry(&entry);

            if *component.status != ChangelistStatus::Deleted {
                if let Some(file_entity) = component.file_entity.get(&client) {
                    if let Ok(mut fs_state) = fs_state_q.get_mut(file_entity) {
                        fs_state.change_status = None;
                    }
                }
            }
        }
        for (entity, component) in events.read::<FileDependency>() {
            info!("entity: `{:?}`, removed FileDependency", entity);

            let file_entity = component.file_entity.get(&client).unwrap();
            let dependency_entity = component.dependency_entity.get(&client).unwrap();

            file_manager.file_remove_dependency(&file_entity, &dependency_entity);
        }
        for (vertex_entity_3d, _) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, removed Vertex3d", vertex_entity_3d);

            vertex_manager.cleanup_deleted_vertex(
                &mut commands,
                &mut canvas,
                &mut input_manager,
                &vertex_entity_3d,
            );
        }
        for (edge_3d_entity, _) in events.read::<Edge3d>() {
            info!("entity: `{:?}`, removed Edge3d", edge_3d_entity);

            edge_manager.cleanup_deleted_edge(
                &mut commands,
                &mut canvas,
                &mut input_manager,
                &mut vertex_manager,
                &mut face_manager,
                &edge_3d_entity,
            );
        }
        for (face_entity_3d, _) in events.read::<Face3d>() {
            info!("entity: `{:?}`, removed Face3d", face_entity_3d);

            face_manager.cleanup_deleted_face_3d(&mut commands, &mut meshes, &face_entity_3d);
        }
    }
}
