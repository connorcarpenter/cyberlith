use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{
    events::{
        DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
        UpdateComponentEvents,
    },
    Server,
};

use vortex_proto::{
    components::{Face3d,
        Edge3d, FileSystemChild, FileSystemEntry, FileSystemRootChild, FileType, OwnedByFile,
        Vertex3d, VertexRoot,
    },
    resources::FileEntryKey,
};
use crate::files::ShapeType;

use crate::resources::{
    file_waitlist::{fs_process_insert, FSWaitlist, FSWaitlistInsert},
    GitManager, ShapeManager, ShapeWaitlist, ShapeWaitlistInsert, TabManager, UserManager,
};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(_user_key, entity) in event_reader.iter() {
        info!("entity: `{:?}`, spawned", entity);
    }
}

pub fn despawn_entity_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut event_reader: EventReader<DespawnEntityEvent>,
) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        let Some(user_session_data) = user_manager.user_session_data(user_key) else {
            panic!("user not found");
        };
        let project = git_manager
            .project_mut(&user_session_data.project_key().unwrap())
            .unwrap();

        let mut shape_type = None;
        if project.entity_is_file(entity) {
            shape_type = Some((true, None));
        } else if shape_manager.has_vertex(entity) {
            shape_type = Some((false, Some(ShapeType::Vertex)));
        } else if shape_manager.has_edge(entity) {
            shape_type = Some((false, Some(ShapeType::Edge)));
        } else if shape_manager.has_face(entity) {
            shape_type = Some((false, Some(ShapeType::Face)));
        }

        match shape_type {
            Some((true, None)) => {
                // file
                info!("entity: `{:?}` (which is a File), despawned", entity);

                project.on_client_delete_file(&mut commands, &mut server, entity);
            }
            Some((false, Some(ShapeType::Vertex))) => {
                // vertex
                info!("entity: `{:?}` (which is a Vertex), despawned", entity);

                let other_entities_to_despawn =
                    shape_manager.on_delete_vertex(&mut commands, &mut server, entity);

                git_manager.on_client_remove_content_entity(&entity);
                for other_entity in other_entities_to_despawn {
                    git_manager.on_client_remove_content_entity(&other_entity);
                }
            }
            Some((false, Some(ShapeType::Edge))) => {
                // edge
                info!("entity: `{:?}` (which is an Edge), despawned", entity);

                shape_manager.on_delete_edge(entity);

                git_manager.on_client_remove_content_entity(&entity);
            }
            Some((false, Some(ShapeType::Face))) => {
                // edge
                info!("entity: `{:?}` (which is an Face), despawned", entity);

                shape_manager.on_delete_face(entity);

                git_manager.on_client_remove_content_entity(&entity);
            }
            _ => {
                panic!(
                    "despawned entity: `{:?}` which is ({:?})",
                    entity, shape_type
                );
            }
        }
    }
}

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
            info!("entity: `{:?}`, inserted Face3d", face_entity);

            shape_waitlist.process_insert(
                &mut server,
                &mut git_manager,
                &mut shape_manager,
                ShapeWaitlistInsert::Face(face_entity),
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

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (_user_key, _entity, _component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");
            // TODO!
        }
        for (_user_key, _entity, _component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");
            // TODO!
        }
        // on Vertex3D Remove Event
        for (_user_key, entity, _component) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, removed Vertex3d", entity);
        }
        // on Edge3d Remove Event
        for (_user_key, entity, _) in events.read::<Edge3d>() {
            info!("entity: `{:?}`, removed Edge3d", entity);
        }
        // on VertexRoot Remove Event
        for (_, entity, _) in events.read::<VertexRoot>() {
            panic!(
                "entity: `{:?}`, removed VertexRoot, how is this possible?",
                entity
            );
        }
    }
}

pub fn update_component_events(
    mut event_reader: EventReader<UpdateComponentEvents>,
    mut commands: Commands,
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_user_key, _entity) in events.read::<FileSystemEntry>() {
            // TODO!
        }
        // on FileSystemChild Update Event
        for (_user_key, _entity) in events.read::<FileSystemChild>() {
            // TODO!
        }
        // on Vertex3D Update Event
        for (_, entity) in events.read::<Vertex3d>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
    }
}
