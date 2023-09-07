use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{events::InsertComponentEvents, Server};

use vortex_proto::{
    components::{
        Edge3d, Face3d, FileSystemChild, FileSystemEntry, FileSystemRootChild, FileType,
        OwnedByFile, Vertex3d, VertexRoot, ShapeName,
    },
    resources::FileEntryKey,
};

use crate::{resources::{
    file_waitlist::{file_process_insert, FSWaitlist, FSWaitlistInsert},
    GitManager, ShapeManager, ShapeWaitlist, ShapeWaitlistInsert, TabManager, UserManager,
}, events::InsertComponentEvent};

pub fn insert_component_events(
    mut event_reader: EventReader<InsertComponentEvents>,

    // for filesystem
    mut insert_fs_entry_event_writer: EventWriter<InsertComponentEvent<FileSystemEntry>>,
    mut insert_fs_root_event_writer: EventWriter<InsertComponentEvent<FileSystemRootChild>>,
    mut insert_fs_child_event_writer: EventWriter<InsertComponentEvent<FileSystemChild>>,

    // for vertices
    mut insert_vertex_3d_event_writer: EventWriter<InsertComponentEvent<Vertex3d>>,
    mut insert_vertex_root_event_writer: EventWriter<InsertComponentEvent<VertexRoot>>,

    mut insert_edge_3d_event_writer: EventWriter<InsertComponentEvent<Edge3d>>,
    mut insert_face_3d_event_writer: EventWriter<InsertComponentEvent<Face3d>>,
    mut insert_file_type_event_writer: EventWriter<InsertComponentEvent<FileType>>,
    mut insert_owned_by_event_writer: EventWriter<InsertComponentEvent<OwnedByFile>>,
    mut insert_shape_name_event_writer: EventWriter<InsertComponentEvent<ShapeName>>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {
            insert_fs_entry_event_writer.send(InsertComponentEvent::<FileSystemEntry>::new(user_key, entity));
        }

        // on FileSystemRootChild Insert Event
        for (user_key, entity) in events.read::<FileSystemRootChild>() {
            insert_fs_root_event_writer
                .send(InsertComponentEvent::<FileSystemRootChild>::new(user_key, entity));
        }

        // on FileSystemChild Insert Event
        for (user_key, entity) in events.read::<FileSystemChild>() {
            insert_fs_child_event_writer.send(InsertComponentEvent::<FileSystemChild>::new(user_key, entity));
        }

        // on Vertex3d Insert Event
        for (user_key, entity) in events.read::<Vertex3d>() {
            insert_vertex_3d_event_writer.send(InsertComponentEvent::<Vertex3d>::new(user_key, entity));
        }

        // on Vertex Root Child Event
        for (user_key, entity) in events.read::<VertexRoot>() {
            insert_vertex_root_event_writer.send(InsertComponentEvent::<VertexRoot>::new(user_key, entity));
        }

        // on OwnedByFile Insert Event
        for (user_key, entity) in events.read::<OwnedByFile>() {
            insert_owned_by_event_writer.send(InsertComponentEvent::<OwnedByFile>::new(user_key, entity));
        }

        // on FileType Insert Event
        for (user_key, entity) in events.read::<FileType>() {
            insert_file_type_event_writer.send(InsertComponentEvent::<FileType>::new(user_key, entity));
        }

        // on Edge3d Insert Event
        for (user_key, entity) in events.read::<Edge3d>() {
            insert_edge_3d_event_writer.send(InsertComponentEvent::<Edge3d>::new(user_key, entity));
        }

        // on Face3d Insert Event
        for (user_key, entity) in events.read::<Face3d>() {
            insert_face_3d_event_writer.send(InsertComponentEvent::<Face3d>::new(user_key, entity));
        }

        // on ShapeName Insert Event
        for (user_key, entity) in events.read::<ShapeName>() {
            insert_shape_name_event_writer.send(InsertComponentEvent::<ShapeName>::new(user_key, entity));
        }
    }
}

pub fn insert_file_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    mut fs_waiting_entities: Local<HashMap<Entity, FSWaitlist>>,
    mut fs_entry_events: EventReader<InsertComponentEvent<FileSystemEntry>>,
    mut fs_child_events: EventReader<InsertComponentEvent<FileSystemChild>>,
    mut fs_root_child_events: EventReader<InsertComponentEvent<FileSystemRootChild>>,
    fs_entry_q: Query<&FileSystemEntry>,
    fs_child_q: Query<&FileSystemChild>,
    entry_key_q: Query<&FileEntryKey>,
) {
    // on FileSystemEntry Insert Event
    for event in fs_entry_events.iter() {
        let user_key = event.user_key;
        let entity = event.entity;
        info!("inserted FileSystemEntry");
        let entry = fs_entry_q.get(entity).unwrap();
        file_process_insert(
            &mut commands,
            &mut server,
            FSWaitlistInsert::Entry(*entry.kind, (*entry.name).clone()),
            &user_manager,
            &mut git_manager,
            &mut tab_manager,
            &mut fs_waiting_entities,
            &user_key,
            &entity,
        );
    }

    // on FileSystemRootChild Insert Event
    for event in fs_root_child_events.iter() {
        let user_key = event.user_key;
        let entity = event.entity;
        info!("inserted FileSystemRootChild");
        file_process_insert(
            &mut commands,
            &mut server,
            FSWaitlistInsert::Parent(None),
            &user_manager,
            &mut git_manager,
            &mut tab_manager,
            &mut fs_waiting_entities,
            &user_key,
            &entity,
        );
    }

    // on FileSystemChild Insert Event
    for event in fs_child_events.iter() {
        let user_key = event.user_key;
        let entity = event.entity;
        info!("inserted FileSystemChild");
        let entry = fs_child_q.get(entity).unwrap();
        let Some(parent_entity) = entry.parent_id.get(&server) else {
            panic!("no parent entity!")
        };
        let parent_key = entry_key_q.get(parent_entity).unwrap();
        file_process_insert(
            &mut commands,
            &mut server,
            FSWaitlistInsert::Parent(Some(parent_key.clone())),
            &user_manager,
            &mut git_manager,
            &mut tab_manager,
            &mut fs_waiting_entities,
            &user_key,
            &entity,
        );
    }
}

pub fn insert_vertex_component_events(
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut vert_3d_events: EventReader<InsertComponentEvent<Vertex3d>>,
    mut vert_root_events: EventReader<InsertComponentEvent<VertexRoot>>,
) {
    // on Vertex3D Insert Event
    for event in vert_3d_events.iter() {
        let entity = event.entity;
        info!("entity: `{:?}`, inserted Vertex3d", entity);

        shape_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut shape_manager,
            ShapeWaitlistInsert::Vertex(entity),
        );
    }

    // on VertexRoot Insert Event
    for event in vert_root_events.iter() {
        let entity = event.entity;
        info!("entity: `{:?}`, inserted VertexRoot", entity);

        shape_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut shape_manager,
            ShapeWaitlistInsert::VertexRoot(entity),
        );
    }
}

pub fn insert_edge_component_events(
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut edge_3d_events: EventReader<InsertComponentEvent<Edge3d>>,
    edge_3d_q: Query<&Edge3d>,
) {
    // on Edge3d Insert Event
    for event in edge_3d_events.iter() {
        let edge_entity = event.entity;
        info!("entity: `{:?}`, inserted Edge3d", edge_entity);

        let edge_3d = edge_3d_q.get(edge_entity).unwrap();
        let Some(start_entity) = edge_3d.start.get(&server) else {
            panic!("no parent entity!")
        };
        let Some(end_entity) = edge_3d.end.get(&server) else {
            panic!("no child entity!")
        };
        shape_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut shape_manager,
            ShapeWaitlistInsert::Edge(start_entity, edge_entity, end_entity),
        );
    }
}

pub fn insert_face_component_events(
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut face_3d_events: EventReader<InsertComponentEvent<Face3d>>,
    face_3d_q: Query<&Face3d>,
) {
    // on Face3d Insert Event
    for event in face_3d_events.iter() {
        let face_entity = event.entity;
        let face_3d = face_3d_q.get(face_entity).unwrap();

        let vertex_a = face_3d.vertex_a.get(&server).unwrap();
        let vertex_b = face_3d.vertex_b.get(&server).unwrap();
        let vertex_c = face_3d.vertex_c.get(&server).unwrap();
        let edge_a = face_3d.edge_a.get(&server).unwrap();
        let edge_b = face_3d.edge_b.get(&server).unwrap();
        let edge_c = face_3d.edge_c.get(&server).unwrap();

        info!(
            "entity: `{:?}`, inserted Face3d(vertices({:?}, {:?}, {:?}), edges({:?}, {:?}, {:?}))",
            face_entity,
            vertex_a, vertex_b, vertex_c,
            edge_a, edge_b, edge_c
        );

        shape_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut shape_manager,
            ShapeWaitlistInsert::Face(
                face_entity,
                vertex_a,
                vertex_b,
                vertex_c,
                edge_a,
                edge_b,
                edge_c,
            ),
        );
    }
}

pub fn insert_shape_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut file_type_events: EventReader<InsertComponentEvent<FileType>>,
    mut owned_by_file_events: EventReader<InsertComponentEvent<OwnedByFile>>,
    mut shape_name_events: EventReader<InsertComponentEvent<ShapeName>>,
    entry_key_q: Query<&FileEntryKey>,
    file_type_q: Query<&FileType>,
    owned_by_file_q: Query<&OwnedByFile>,
    shape_name_q: Query<&ShapeName>,
) {
    // on Shape FileType Insert Event
    for event in file_type_events.iter() {
        let entity = event.entity;

        let file_type_value = *file_type_q.get(entity).unwrap().value;

        info!(
            "entity: `{:?}`, inserted FileType: {:?}",
            entity, file_type_value
        );

        shape_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut shape_manager,
            ShapeWaitlistInsert::FileType(entity, file_type_value),
        );
    }

    // on OwnedByFile Insert Event
    for event in owned_by_file_events.iter() {
        let user_key = event.user_key;
        let entity = event.entity;
        let file_entity = owned_by_file_q
            .get(entity)
            .unwrap()
            .file_entity
            .get(&server)
            .unwrap();
        let file_key = entry_key_q.get(file_entity).unwrap();
        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();

        info!(
            "entity: `{:?}`, inserted OwnedByFile({:?})",
            entity, file_entity
        );

        shape_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut shape_manager,
            ShapeWaitlistInsert::OwnedByFile(entity, project_key, file_key.clone()),
        );
    }

    // on ShapeName Insert Event
    for event in shape_name_events.iter() {
        let entity = event.entity;
        let shape_name = (*shape_name_q.get(entity).unwrap().value).clone();

        info!(
            "entity: `{:?}`, inserted ShapeName: {:?}",
            entity, shape_name
        );

        let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
            panic!("no content entity keys!");
        };
        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
}