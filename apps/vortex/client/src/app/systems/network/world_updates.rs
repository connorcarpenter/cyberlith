use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::{info, warn};

use naia_bevy_client::{
    events::{
        DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
        UpdateComponentEvents,
    },
    Client,
};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::{
    ChangelistEntry, Edge3d, EntryKind, Face3d, FileSystemChild, FileSystemEntry,
    FileSystemRootChild, FileType, OwnedByFile, Vertex3d, VertexRoot,
};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
    events::InsertComponentEvent,
    resources::{
        camera_manager::CameraManager,
        global::Global,
        shape_manager::ShapeManager,
        shape_waitlist::{ShapeWaitlist, ShapeWaitlistInsert},
    },
    systems::file_post_process,
};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(entity) in event_reader.iter() {
        info!("entity: `{:?}`, spawned", entity);
    }
}

pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent>) {
    for DespawnEntityEvent(entity) in event_reader.iter() {
        info!("entity: `{:?}`, despawned", entity);
    }
}

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

        // on Face3d Insert Event
        for entity in events.read::<Face3d>() {
            insert_face_3d_event_writer.send(InsertComponentEvent::<Face3d>::new(entity));
        }
    }
}

pub fn insert_fs_component_events(
    mut commands: Commands,
    global: Res<Global>,
    client: Client,
    mut entry_events: EventReader<InsertComponentEvent<FileSystemEntry>>,
    mut root_events: EventReader<InsertComponentEvent<FileSystemRootChild>>,
    mut child_events: EventReader<InsertComponentEvent<FileSystemChild>>,
    entry_q: Query<&FileSystemEntry>,
    mut parent_q: Query<&mut FileSystemParent>,
    child_q: Query<&FileSystemChild>,
) {
    let project_root_entity = global.project_root_entity;
    let mut recent_parents: Option<HashMap<Entity, FileSystemParent>> = None;

    for event in entry_events.iter() {
        let entity = event.entity;
        let entry = entry_q.get(entity).unwrap();
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
    mut global: ResMut<Global>,
    client: Client,
    mut events: EventReader<InsertComponentEvent<ChangelistEntry>>,
    changelist_q: Query<&ChangelistEntry>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    // on ChangelistEntry Insert Event
    for event in events.iter() {
        let entity = event.entity;
        commands.entity(entity).insert(ChangelistUiState::new());

        let entry = changelist_q.get(entity).unwrap();

        // associate status with file entry
        if let Some(file_entity) = entry.file_entity.get(&client) {
            let mut fs_state = fs_state_q.get_mut(file_entity).unwrap();
            fs_state.change_status = Some(*entry.status);
        }

        // insert into changelist resource
        global.changelist.insert(entry.file_entry_key(), entity);

        info!(
            "Received ChangelistEntry insert event. entity: `{:?}`, path: `{:?}`, name: `{:?}`",
            entity, *entry.path, *entry.name
        );
    }
}

pub fn insert_vertex_events(
    mut commands: Commands,
    mut vertex_3d_events: EventReader<InsertComponentEvent<Vertex3d>>,
    mut vertex_root_events: EventReader<InsertComponentEvent<VertexRoot>>,

    mut camera_manager: ResMut<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,
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
            &mut shape_manager,
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
            &mut shape_manager,
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

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    edge_3d_q: Query<&Edge3d>,
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
            &mut shape_manager,
            &transform_q,
            &edge_entity,
            ShapeWaitlistInsert::Edge(start_entity, end_entity),
        );
    }
}

pub fn insert_face_events(
    mut commands: Commands,
    client: Client,
    mut face_3d_events: EventReader<InsertComponentEvent<Face3d>>,
    mut camera_manager: ResMut<CameraManager>,
    mut shape_manager: ResMut<ShapeManager>,
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
            &mut shape_manager,
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
    mut shape_manager: ResMut<ShapeManager>,
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
            &mut shape_manager,
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
            &mut shape_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::FileType(file_type_value),
        );
    }
}

pub fn update_component_events(
    mut event_reader: EventReader<UpdateComponentEvents>,
    entry_query: Query<&FileSystemEntry>,
    mut shape_manager: ResMut<ShapeManager>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_, entry_entity) in events.read::<FileSystemEntry>() {
            let entry = entry_query.get(entry_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemEntry: `{:?}` ({:?})",
                entry_entity, entry_name
            );
        }
        // on FileSystemRootChild Update Event
        for (_, child_entity) in events.read::<FileSystemRootChild>() {
            let entry = entry_query.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemRootChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on FileSystemChild Update Event
        for (_, child_entity) in events.read::<FileSystemChild>() {
            let entry = entry_query.get(child_entity).unwrap();
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

pub fn remove_component_events(
    mut commands: Commands,
    client: Client,
    mut global: ResMut<Global>,
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

            let Ok(mut parent) = parent_q.get_mut(global.project_root_entity) else {
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
            global.changelist.remove(&entry);

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
