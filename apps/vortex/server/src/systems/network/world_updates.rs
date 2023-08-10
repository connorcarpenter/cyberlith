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
    components::{
        FileType, Edge3d, FileSystemChild, FileSystemEntry, FileSystemRootChild, Vertex3d, VertexRoot,
    },
    resources::FileEntryKey,
};

use crate::{
    files::handle_file_modify,
    resources::{
        fs_waitlist::{fs_process_insert, FSWaitlist, FSWaitlistInsert},
        GitManager, TabManager, UserManager, VertexManager, VertexWaitlist, VertexWaitlistInsert,
    },
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
    mut vertex_manager: ResMut<VertexManager>,
    mut event_reader: EventReader<DespawnEntityEvent>,
) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        let Some(user) = user_manager.user_info(user_key) else {
            panic!("user not found");
        };
        let workspace = git_manager.workspace_mut(user.get_username());

        let entity_is_file = workspace.entity_is_file(entity);
        let entity_is_vertex = vertex_manager.has_vertex(entity);

        match (entity_is_file, entity_is_vertex) {
            (true, true) => {
                panic!("entity is both file and vertex");
            }
            (false, false) => {
                panic!("entity is neither file nor vertex");
            }
            (true, false) => {
                // file
                info!("entity: `{:?}` (which is a File), despawned", entity);
            }
            (false, true) => {
                // vertex
                info!("entity: `{:?}` (which is a Vertex), despawned", entity);
            }
        }

        if entity_is_file {
            workspace.on_client_delete_file(&mut commands, &mut server, entity);
        } else if entity_is_vertex {
            vertex_manager.on_delete_vertex(&mut commands, &mut server, entity);
        }
    }
}

pub fn insert_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    mut vertex_waitlist: ResMut<VertexWaitlist>,
    mut vertex_manager: ResMut<VertexManager>,
    mut fs_waiting_entities: Local<HashMap<Entity, FSWaitlist>>,
    mut event_reader: EventReader<InsertComponentEvents>,
    fs_entry_q: Query<&FileSystemEntry>,
    fs_child_q: Query<&FileSystemChild>,
    entry_key_q: Query<&FileEntryKey>,
    edge_3d_q: Query<&Edge3d>,
    vert_file_type_q: Query<&FileType>,
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
        for (user_key, entity) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, inserted Vertex3d", entity);
            tab_manager.on_insert_vertex(&user_key, &entity);
            handle_file_modify(
                &mut commands,
                &mut server,
                &user_manager,
                &mut git_manager,
                &mut tab_manager,
                &user_key,
                &entity,
                &entry_key_q,
            );
        }

        // on Edge3d Insert Event
        for (_user_key, edge_entity) in events.read::<Edge3d>() {
            info!("entity: `{:?}`, inserted Edge3d", edge_entity);
            let edge_3d = edge_3d_q.get(edge_entity).unwrap();
            let Some(start_entity) = edge_3d.start.get(&server) else {
                panic!("no parent entity!")
            };
            let Some(end_entity) = edge_3d.end.get(&server) else {
                panic!("no child entity!")
            };
            vertex_waitlist.process_insert(
                &mut vertex_manager,
                VertexWaitlistInsert::Edge(start_entity, edge_entity, end_entity),
            );
        }

        // on Vertex FileType Insert Event
        for (_user_key, entity) in events.read::<FileType>() {
            info!("vertex entity: `{:?}`, inserted FileType", entity);
            let file_type_value = *vert_file_type_q.get(entity).unwrap().value;

            vertex_waitlist.process_insert(
                &mut vertex_manager,
                VertexWaitlistInsert::FileType(entity, file_type_value),
            );
        }

        // on VertexRoot Insert Event
        for (_user_key, entity) in events.read::<VertexRoot>() {
            panic!("how is this possible?");
            // info!("entity: `{:?}`, inserted VertexRoot", entity);
            // vertex_manager.on_create_vertex(&entity, None);
            // vertex_manager.finalize_vertex_creation();
        }
    }
}

pub fn remove_component_events(
    mut event_reader: EventReader<RemoveComponentEvents>,
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    entry_key_query: Query<&FileEntryKey>,
) {
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
        for (user_key, entity, _component) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, removed Vertex3d", entity);

            handle_file_modify(
                &mut commands,
                &mut server,
                &user_manager,
                &mut git_manager,
                &mut tab_manager,
                &user_key,
                &entity,
                &entry_key_query,
            );

            tab_manager.on_remove_vertex(&user_key, &entity);
        }
        // on Edge3d Remove Event
        for (_, entity, _) in events.read::<Edge3d>() {
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
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut tab_manager: ResMut<TabManager>,
    entry_key_query: Query<&FileEntryKey>,
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
        for (user_key, entity) in events.read::<Vertex3d>() {
            handle_file_modify(
                &mut commands,
                &mut server,
                &user_manager,
                &mut git_manager,
                &mut tab_manager,
                &user_key,
                &entity,
                &entry_key_query,
            );
        }
    }
}
