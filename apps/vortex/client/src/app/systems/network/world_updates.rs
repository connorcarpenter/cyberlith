use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Commands, Local, Query, Res, ResMut},
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
    Assets,
};
use vortex_proto::components::{
    ChangelistEntry, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild, OwnedByTab,
    Vertex3d, VertexChild, VertexRootChild, VertexType,
};

use crate::app::{
    components::{
        file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
        Edge2dLocal, Edge3dLocal,
    },
    events::InsertComponentEvent,
    resources::{camera_manager::CameraManager, global::Global, vertex_manager::VertexManager},
    systems::{
        file_post_process,
        network::{
            vertex_waitlist::{vertex_process_insert, VertexWaitlistInsert},
            VertexWaitlist,
        },
    },
};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(entity) in event_reader.iter() {
        info!("spawned entity: {:?}", entity);
    }
}

pub fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent>) {
    for DespawnEntityEvent(entity) in event_reader.iter() {
        info!("despawned entity: {:?}", entity);
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
    mut insert_vertex_child_event_writer: EventWriter<InsertComponentEvent<VertexChild>>,
    mut insert_vertex_root_event_writer: EventWriter<InsertComponentEvent<VertexRootChild>>,
    mut insert_owned_by_event_writer: EventWriter<InsertComponentEvent<OwnedByTab>>,
    mut insert_vertex_type_event_writer: EventWriter<InsertComponentEvent<VertexType>>,
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

        // on Vertex Insert Event
        for entity in events.read::<Vertex3d>() {
            insert_vertex_3d_event_writer.send(InsertComponentEvent::<Vertex3d>::new(entity));
        }

        // on Vertex Child Insert Event
        for entity in events.read::<VertexChild>() {
            insert_vertex_child_event_writer.send(InsertComponentEvent::<VertexChild>::new(entity));
        }

        // on Vertex Root Child Event
        for entity in events.read::<VertexRootChild>() {
            insert_vertex_root_event_writer
                .send(InsertComponentEvent::<VertexRootChild>::new(entity));
        }

        // on OwnedByTab Insert Event
        for entity in events.read::<OwnedByTab>() {
            insert_owned_by_event_writer.send(InsertComponentEvent::<OwnedByTab>::new(entity));
        }

        // on Vertex Type Insert Event
        for entity in events.read::<VertexType>() {
            insert_vertex_type_event_writer.send(InsertComponentEvent::<VertexType>::new(entity));
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
            "Received ChangelistEntry insert event. path: `{:?}`, name: `{:?}`",
            *entry.path, *entry.name
        );
    }
}

pub fn insert_vertex_events(
    mut commands: Commands,
    client: Client,
    mut vertex_3d_events: EventReader<InsertComponentEvent<Vertex3d>>,
    mut vertex_root_events: EventReader<InsertComponentEvent<VertexRootChild>>,
    mut vertex_child_events: EventReader<InsertComponentEvent<VertexChild>>,
    mut owned_by_events: EventReader<InsertComponentEvent<OwnedByTab>>,
    mut vertex_type_events: EventReader<InsertComponentEvent<VertexType>>,

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    vertex_child_q: Query<&VertexChild>,
    owned_by_tab_q: Query<&OwnedByTab>,
    vertex_type_q: Query<&VertexType>,
    mut waiting_vertices: Local<VertexWaitlist>,
) {
    // on Vertex Insert Event
    for event in vertex_3d_events.iter() {
        let entity = event.entity;
        vertex_process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut vertex_manager,
            &mut waiting_vertices,
            &entity,
            VertexWaitlistInsert::Position,
        );
    }

    // on Vertex Child Insert Event
    for event in vertex_child_events.iter() {
        let entity = event.entity;
        let vertex_child = vertex_child_q.get(entity).unwrap();
        let Some(parent_entity) = vertex_child.parent_id.get(&client) else {
            warn!("VertexChild component of entity: `{:?}` has no parent component", entity);
            continue;
        };

        vertex_process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut vertex_manager,
            &mut waiting_vertices,
            &entity,
            VertexWaitlistInsert::Parent(Some(parent_entity)),
        );
    }

    // on Vertex Root Child Event
    for event in vertex_root_events.iter() {
        let entity = event.entity;
        vertex_process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut vertex_manager,
            &mut waiting_vertices,
            &entity,
            VertexWaitlistInsert::Parent(None),
        );
    }

    // on OwnedByTab Insert Event
    for event in owned_by_events.iter() {
        let entity = event.entity;
        let owned_by_tab = owned_by_tab_q.get(entity).unwrap();
        let tab_id = *owned_by_tab.tab_id;

        vertex_process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut vertex_manager,
            &mut waiting_vertices,
            &entity,
            VertexWaitlistInsert::OwnedByTab(tab_id),
        );
    }

    // on VertexType Insert Event
    for event in vertex_type_events.iter() {
        let entity = event.entity;
        let vertex_type = *vertex_type_q.get(entity).unwrap().value;

        vertex_process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut vertex_manager,
            &mut waiting_vertices,
            &entity,
            VertexWaitlistInsert::Type(vertex_type),
        );
    }
}

pub fn update_component_events(
    mut event_reader: EventReader<UpdateComponentEvents>,
    entry_query: Query<&FileSystemEntry>,
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
    }
}

pub fn remove_component_events(
    mut commands: Commands,
    client: Client,
    mut global: ResMut<Global>,
    mut event_reader: EventReader<RemoveComponentEvents>,
    mut vertex_manager: ResMut<VertexManager>,
    mut parent_q: Query<&mut FileSystemParent>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
    edge_2d_q: Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: Query<(Entity, &Edge3dLocal)>,
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
        for (vertex_3d_entity, _) in events.read::<Vertex3d>() {
            info!(
                "removed Vertex3d component from entity: {:?}",
                vertex_3d_entity
            );

            vertex_manager.cleanup_deleted_vertex(
                &vertex_3d_entity,
                &mut commands,
                &edge_2d_q,
                &edge_3d_q,
            );
        }
    }
}
