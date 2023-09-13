use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Commands, Query, ResMut},
};
use bevy_log::{info, warn};

use naia_bevy_client::{events::InsertComponentEvents, Client};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::{
    ChangelistEntry, ChangelistStatus, Edge3d, EdgeAngle, EntryKind, Face3d, FileSystemChild,
    FileSystemEntry, FileSystemRootChild, FileType, OwnedByFile, Vertex3d, VertexRoot,
};

use crate::app::{
    components::file_system::{
        ChangelistUiState, FileSystemEntryLocal, FileSystemParent, FileSystemUiState,
    },
    events::InsertComponentEvent,
    resources::{
        camera_manager::CameraManager,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        file_manager::{get_full_path, FileManager},
        shape_manager::ShapeManager,
        shape_waitlist::{ShapeWaitlist, ShapeWaitlistInsert},
        vertex_manager::VertexManager,
    },
    systems::file_post_process,
};
use crate::app::resources::canvas::Canvas;

pub fn insert_component_events(
    mut event_reader: EventReader<InsertComponentEvents>,

    // for filesystem
    mut insert_fs_entry_event_writer: EventWriter<InsertComponentEvent<FileSystemEntry>>,
    mut insert_fs_root_event_writer: EventWriter<InsertComponentEvent<FileSystemRootChild>>,
    mut insert_fs_child_event_writer: EventWriter<InsertComponentEvent<FileSystemChild>>,
    mut insert_cl_entry_event_writer: EventWriter<InsertComponentEvent<ChangelistEntry>>,

    // for vertices
    mut insert_vertex_3d_event_writer: EventWriter<InsertComponentEvent<Vertex3d>>,
    mut insert_vertex_root_event_writer: EventWriter<InsertComponentEvent<VertexRoot>>,
    mut insert_owned_by_event_writer: EventWriter<InsertComponentEvent<OwnedByFile>>,
    mut insert_file_type_event_writer: EventWriter<InsertComponentEvent<FileType>>,
    mut insert_edge_3d_event_writer: EventWriter<InsertComponentEvent<Edge3d>>,
    mut insert_edge_angle_event_writer: EventWriter<InsertComponentEvent<EdgeAngle>>,
    mut insert_face_3d_event_writer: EventWriter<InsertComponentEvent<Face3d>>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for entity in events.read::<FileSystemEntry>() {
            insert_fs_entry_event_writer.send(InsertComponentEvent::<FileSystemEntry>::new(entity));
        }

        // on FileSystemRootChild Insert Event
        for entity in events.read::<FileSystemRootChild>() {
            insert_fs_root_event_writer
                .send(InsertComponentEvent::<FileSystemRootChild>::new(entity));
        }

        // on FileSystemChild Insert Event
        for entity in events.read::<FileSystemChild>() {
            insert_fs_child_event_writer.send(InsertComponentEvent::<FileSystemChild>::new(entity));
        }

        // on ChangelistEntry Insert Event
        for entity in events.read::<ChangelistEntry>() {
            insert_cl_entry_event_writer.send(InsertComponentEvent::<ChangelistEntry>::new(entity));
        }

        // on Vertex3d Insert Event
        for entity in events.read::<Vertex3d>() {
            insert_vertex_3d_event_writer.send(InsertComponentEvent::<Vertex3d>::new(entity));
        }

        // on Vertex Root Child Event
        for entity in events.read::<VertexRoot>() {
            insert_vertex_root_event_writer.send(InsertComponentEvent::<VertexRoot>::new(entity));
        }

        // on OwnedByFile Insert Event
        for entity in events.read::<OwnedByFile>() {
            insert_owned_by_event_writer.send(InsertComponentEvent::<OwnedByFile>::new(entity));
        }

        // on FileType Insert Event
        for entity in events.read::<FileType>() {
            insert_file_type_event_writer.send(InsertComponentEvent::<FileType>::new(entity));
        }

        // on Edge3d Insert Event
        for entity in events.read::<Edge3d>() {
            insert_edge_3d_event_writer.send(InsertComponentEvent::<Edge3d>::new(entity));
        }

        // on EdgeAngle Insert Event
        for entity in events.read::<EdgeAngle>() {
            insert_edge_angle_event_writer.send(InsertComponentEvent::<EdgeAngle>::new(entity));
        }

        // on Face3d Insert Event
        for entity in events.read::<Face3d>() {
            insert_face_3d_event_writer.send(InsertComponentEvent::<Face3d>::new(entity));
        }
    }
}

pub fn insert_file_component_events(
    mut commands: Commands,
    mut file_manager: ResMut<FileManager>,
    client: Client,
    mut entry_events: EventReader<InsertComponentEvent<FileSystemEntry>>,
    mut root_events: EventReader<InsertComponentEvent<FileSystemRootChild>>,
    mut child_events: EventReader<InsertComponentEvent<FileSystemChild>>,
    entry_q: Query<&FileSystemEntry>,
    mut parent_q: Query<&mut FileSystemParent>,
    child_q: Query<&FileSystemChild>,
) {
    let project_root_entity = file_manager.project_root_entity;
    let mut recent_parents: Option<HashMap<Entity, FileSystemParent>> = None;

    for event in entry_events.iter() {
        let entity = event.entity;
        let entry = entry_q.get(entity).unwrap();

        info!(
            "entity: `{:?}` - inserted FileSystemEntry. kind: `{:?}`, name: `{:?}`",
            entity, *entry.kind, *entry.name
        );

        file_post_process::insert_ui_state_component(&mut commands, entity, false);
        if *entry.kind == EntryKind::Directory {
            if recent_parents.is_none() {
                recent_parents = Some(HashMap::new());
            }
            recent_parents
                .as_mut()
                .unwrap()
                .insert(entity, FileSystemParent::new());
        }
        file_manager.on_file_create(&entity);
        commands
            .entity(entity)
            .insert(FileSystemEntryLocal::new(&*entry.name));
    }

    for event in root_events.iter() {
        let entity = event.entity;
        // Add children to root parent
        let entry = entry_q.get(entity).unwrap();
        let mut parent = parent_q.get_mut(project_root_entity).unwrap();
        file_post_process::parent_add_child_entry(&mut parent, entry, entity);
    }

    for event in child_events.iter() {
        let entity = event.entity;
        let entry = entry_q.get(entity).unwrap();
        // Get parent
        let Some(parent_entity) = child_q
                .get(entity)
                .unwrap()
                .parent_id
                .get(&client) else {
                panic!("FileSystemChild component of entry: `{}` has no parent component", *entry.name);
            };

        if let Ok(mut parent) = parent_q.get_mut(parent_entity) {
            file_post_process::parent_add_child_entry(&mut parent, entry, entity);
        } else {
            let Some(parent_map) = recent_parents.as_mut() else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", entity, parent_entity);
                };
            let Some(parent) = parent_map.get_mut(&parent_entity) else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", entity, parent_entity);
                };
            file_post_process::parent_add_child_entry(parent, entry, entity);
        }
    }

    // Add all parents now that the children were able to process
    // Note that we do it this way because Commands aren't flushed till the end of the system
    if let Some(parent_map) = recent_parents.as_mut() {
        for (entity, parent) in parent_map.drain() {
            commands.entity(entity).insert(parent);
        }
    }
}

pub fn insert_changelist_entry_events(
    mut commands: Commands,
    mut file_manager: ResMut<FileManager>,
    client: Client,
    mut events: EventReader<InsertComponentEvent<ChangelistEntry>>,
    changelist_q: Query<&ChangelistEntry>,
    fs_q: Query<(&FileSystemEntry, Option<&FileSystemChild>)>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    // on ChangelistEntry Insert Event
    for event in events.iter() {
        let cl_entity = event.entity;

        let entry = changelist_q.get(cl_entity).unwrap();

        info!(
            "entity: `{:?}` - inserted ChangelistEntry. path: `{:?}`, name: `{:?}`",
            cl_entity, *entry.path, *entry.name
        );

        let mut display_name: String = (*entry.name).clone();
        let mut display_path: String = (*entry.path).clone();
        let mut file_entity_opt: Option<Entity> = None;
        let mut parent_entity_opt: Option<Entity> = None;

        // associate status with file entry
        if *entry.status != ChangelistStatus::Deleted {
            let file_entity = entry.file_entity.get(&client).unwrap();

            let mut fs_state = fs_state_q.get_mut(file_entity).unwrap();
            fs_state.change_status = Some(*entry.status);

            let (fs_entry, fs_child_opt) = fs_q.get(file_entity).unwrap();
            display_name = (*fs_entry.name).clone();
            file_entity_opt = Some(file_entity);

            if let Some(fs_child) = fs_child_opt {
                parent_entity_opt = fs_child.parent_id.get(&client);
            }

            if parent_entity_opt.is_some() {
                let new_display_path = get_full_path(&client, &fs_q, file_entity);
                info!(
                    "change path for entity: `{:?}`. path was: `{:?}`, now is `{:?}`",
                    file_entity, display_path, new_display_path
                );
                display_path = new_display_path;
            }
        }

        // insert ui state component
        commands
            .entity(cl_entity)
            .insert(ChangelistUiState::new(&display_name, &display_path));

        // insert into changelist resource
        file_manager.insert_changelist_entry(
            entry.file_entry_key(),
            file_entity_opt,
            parent_entity_opt,
            cl_entity,
        );
    }
}

pub fn insert_vertex_events(
    mut commands: Commands,
    mut vertex_3d_events: EventReader<InsertComponentEvent<Vertex3d>>,
    mut vertex_root_events: EventReader<InsertComponentEvent<VertexRoot>>,

    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    transform_q: Query<&Transform>,
) {
    // on Vertex Insert Event
    for event in vertex_3d_events.iter() {
        let entity = event.entity;

        info!("entity: {:?} - inserted Vertex3d", entity);

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::Vertex,
        );
    }

    // on Vertex Root Event
    for event in vertex_root_events.iter() {
        let entity = event.entity;

        info!("entity: {:?} - inserted VertexRoot", entity);

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::VertexRoot,
        );
    }
}

pub fn insert_edge_events(
    mut commands: Commands,
    client: Client,
    mut edge_3d_events: EventReader<InsertComponentEvent<Edge3d>>,
    mut edge_angle_events: EventReader<InsertComponentEvent<EdgeAngle>>,

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    edge_3d_q: Query<&Edge3d>,
    edge_angle_q: Query<&EdgeAngle>,
    transform_q: Query<&Transform>,
) {
    // on Edge3d Insert Event
    for event in edge_3d_events.iter() {
        // handle vertex
        let edge_entity = event.entity;

        info!("entity: {:?} - inserted Edge3d", edge_entity);

        let edge_3d = edge_3d_q.get(edge_entity).unwrap();
        let Some(start_entity) = edge_3d.start.get(&client) else {
            warn!("Edge3d component of entity: `{:?}` has no start entity", edge_entity);
            continue;
        };
        let Some(end_entity) = edge_3d.end.get(&client) else {
            warn!("Edge3d component of entity: `{:?}` has no start entity", edge_entity);
            continue;
        };

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &edge_entity,
            ShapeWaitlistInsert::Edge(start_entity, end_entity),
        );
    }

    // on EdgeAngle Insert Event
    for event in edge_angle_events.iter() {
        // handle vertex
        let edge_entity = event.entity;

        info!("entity: {:?} - inserted EdgeAngle", edge_entity);

        let edge_3d = edge_angle_q.get(edge_entity).unwrap();

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &edge_entity,
            ShapeWaitlistInsert::EdgeAngle(edge_3d.get_radians()),
        );
    }
}

pub fn insert_face_events(
    mut commands: Commands,
    client: Client,
    mut face_3d_events: EventReader<InsertComponentEvent<Face3d>>,
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    face_3d_q: Query<&Face3d>,
    transform_q: Query<&Transform>,
) {
    // on Face3d Insert Event
    for event in face_3d_events.iter() {
        // handle face
        let face_entity = event.entity;

        info!("entity: {:?} - inserted Face3d", face_entity);

        let face_3d = face_3d_q.get(face_entity).unwrap();
        let Some(vertex_a_entity) = face_3d.vertex_a.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(vertex_b_entity) = face_3d.vertex_b.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(vertex_c_entity) = face_3d.vertex_c.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(edge_a_entity) = face_3d.edge_a.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(edge_b_entity) = face_3d.edge_b.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(edge_c_entity) = face_3d.edge_c.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &face_entity,
            ShapeWaitlistInsert::Face(
                vertex_a_entity,
                vertex_b_entity,
                vertex_c_entity,
                edge_a_entity,
                edge_b_entity,
                edge_c_entity,
            ),
        );
    }
}

pub fn insert_shape_events(
    mut commands: Commands,
    client: Client,
    mut owned_by_events: EventReader<InsertComponentEvent<OwnedByFile>>,
    mut file_type_events: EventReader<InsertComponentEvent<FileType>>,

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    owned_by_tab_q: Query<&OwnedByFile>,
    file_type_q: Query<&FileType>,
    transform_q: Query<&Transform>,
) {
    // on OwnedByFile Insert Event
    for event in owned_by_events.iter() {
        let entity = event.entity;

        info!("entity: {:?} - inserted OwnedByFile", entity);

        let owned_by_file = owned_by_tab_q.get(entity).unwrap();
        let file_entity = owned_by_file.file_entity.get(&client).unwrap();

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::OwnedByFile(file_entity),
        );
    }

    // on FileType Insert Event
    for event in file_type_events.iter() {
        let entity = event.entity;

        let file_type = file_type_q.get(entity).unwrap();
        let file_type_value = *file_type.value;

        info!(
            "entity: {:?} - inserted FileType::{:?}",
            entity, file_type_value
        );

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::FileType(file_type_value),
        );
    }
}
