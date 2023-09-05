use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{
    events::InsertComponentEvents,
    Server,
};

use vortex_proto::{
    components::{
        Edge3d, Face3d, FileSystemChild, FileSystemEntry, FileSystemRootChild, FileType,
        OwnedByFile, Vertex3d, VertexRoot,
    },
    resources::FileEntryKey,
};

use crate::resources::{
    file_waitlist::{fs_process_insert, FSWaitlist, FSWaitlistInsert},
    GitManager, ShapeManager, ShapeWaitlist, ShapeWaitlistInsert, TabManager, UserManager,
};

pub fn insert_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut fs_waiting_entities: Local<HashMap<Entity, FSWaitlist>>,
    mut event_reader: EventReader<InsertComponentEvents>,
    fs_entry_q: Query<&FileSystemEntry>,
    fs_child_q: Query<&FileSystemChild>,
    entry_key_q: Query<&FileEntryKey>,
    edge_3d_q: Query<&Edge3d>,
    face_3d_q: Query<&Face3d>,
    vert_file_type_q: Query<&FileType>,
    owned_by_file_q: Query<&OwnedByFile>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {
            info!("inserted FileSystemEntry");
            let entry = fs_entry_q.get(entity).unwrap();
            fs_process_insert(
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
        for (user_key, entity) in events.read::<FileSystemRootChild>() {
            info!("inserted FileSystemRootChild");
            fs_process_insert(
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
        for (user_key, entity) in events.read::<FileSystemChild>() {
            info!("inserted FileSystemChild");
            let entry = fs_child_q.get(entity).unwrap();
            let Some(parent_entity) = entry.parent_id.get(&server) else {
                panic!("no parent entity!")
            };
            let parent_key = entry_key_q.get(parent_entity).unwrap();
            fs_process_insert(
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

        // on Vertex3D Insert Event
        for (_, entity) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, inserted Vertex3d", entity);

            shape_waitlist.process_insert(
                &mut server,
                &mut git_manager,
                &mut shape_manager,
                ShapeWaitlistInsert::Vertex(entity),
            );
        }

        // on VertexRoot Insert Event
        for (_, entity) in events.read::<VertexRoot>() {
            info!("entity: `{:?}`, inserted VertexRoot", entity);

            shape_waitlist.process_insert(
                &mut server,
                &mut git_manager,
                &mut shape_manager,
                ShapeWaitlistInsert::VertexRoot(entity),
            );
        }

        // on Edge3d Insert Event
        for (_, edge_entity) in events.read::<Edge3d>() {
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

        // on Face3d Insert Event
        for (_, face_entity) in events.read::<Face3d>() {
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

        // on Shape FileType Insert Event
        for (_user_key, entity) in events.read::<FileType>() {
            let file_type_value = *vert_file_type_q.get(entity).unwrap().value;

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
        for (user_key, entity) in events.read::<OwnedByFile>() {
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

        // on VertexRoot Insert Event
        for (_user_key, _entity) in events.read::<VertexRoot>() {
            panic!("how is this possible?");
            // info!("entity: `{:?}`, inserted VertexRoot", entity);
            // shape_manager.on_create_vertex(&entity, None);
            // shape_manager.finalize_vertex_creation();
        }
    }
}