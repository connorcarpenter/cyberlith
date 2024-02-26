use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Commands, Local, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::info;

use naia_bevy_server::{events::InsertComponentEvents, Replicate, Server};

use editor_proto::{
    components::{
        AnimFrame, AnimRotation, BackgroundSkinColor, Edge3d, Face3d, FaceColor, FileDependency,
        FileExtension, FileSystemChild, FileSystemEntry, FileSystemRootChild, FileType, IconEdge,
        IconFace, IconFrame, IconVertex, NetTransform, OwnedByFile, PaletteColor, ShapeName,
        SkinOrSceneEntity, Vertex3d, VertexRoot,
    },
    resources::FileKey,
};

use crate::{
    events::InsertComponentEvent,
    resources::{
        file_waitlist::{file_process_insert, FSWaitlist, FSWaitlistInsert},
        AnimationManager, ComponentWaitlist, ComponentWaitlistInsert, ContentEntityData,
        GitManager, IconManager, PaletteManager, ShapeManager, SkinManager, TabManager,
        UserManager,
    },
};

#[derive(Resource)]
struct CachedInsertComponentEventsState {
    event_state: SystemState<EventReader<'static, 'static, InsertComponentEvents>>,
}

pub fn insert_component_event_startup(world: &mut World) {
    let initial_state: SystemState<EventReader<InsertComponentEvents>> = SystemState::new(world);
    world.insert_resource(CachedInsertComponentEventsState {
        event_state: initial_state,
    });
}

pub fn insert_component_events(world: &mut World) {
    let mut events_collection: Vec<InsertComponentEvents> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedInsertComponentEventsState>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.read() {
                events_collection.push(events.clone());
            }
        },
    );

    for events in events_collection {
        insert_component_event::<FileSystemEntry>(world, &events);
        insert_component_event::<FileSystemRootChild>(world, &events);
        insert_component_event::<FileSystemChild>(world, &events);
        insert_component_event::<FileDependency>(world, &events);

        insert_component_event::<Vertex3d>(world, &events);
        insert_component_event::<VertexRoot>(world, &events);
        insert_component_event::<Edge3d>(world, &events);
        insert_component_event::<Face3d>(world, &events);

        insert_component_event::<IconVertex>(world, &events);
        insert_component_event::<IconEdge>(world, &events);
        insert_component_event::<IconFace>(world, &events);
        insert_component_event::<IconFrame>(world, &events);

        insert_component_event::<FileType>(world, &events);
        insert_component_event::<OwnedByFile>(world, &events);
        insert_component_event::<ShapeName>(world, &events);
        insert_component_event::<AnimFrame>(world, &events);
        insert_component_event::<AnimRotation>(world, &events);
        insert_component_event::<PaletteColor>(world, &events);
        insert_component_event::<BackgroundSkinColor>(world, &events);
        insert_component_event::<FaceColor>(world, &events);

        insert_component_event::<NetTransform>(world, &events);
        insert_component_event::<SkinOrSceneEntity>(world, &events);
    }
}

fn insert_component_event<T: Replicate>(world: &mut World, events: &InsertComponentEvents) {
    let mut system_state: SystemState<EventWriter<InsertComponentEvent<T>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for (user_key, entity) in events.read::<T>() {
        event_writer.send(InsertComponentEvent::<T>::new(user_key, entity));
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
    mut fs_dependency_events: EventReader<InsertComponentEvent<FileDependency>>,
    fs_entry_q: Query<&FileSystemEntry>,
    fs_child_q: Query<&FileSystemChild>,
    key_q: Query<&FileKey>,
    fs_dependency_q: Query<&FileDependency>,
) {
    // on FileSystemEntry Insert Event
    for event in fs_entry_events.read() {
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
    for event in fs_root_child_events.read() {
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
    for event in fs_child_events.read() {
        let user_key = event.user_key;
        let entity = event.entity;
        info!("entity `{:?}` inserted FileSystemChild", entity);
        let entry = fs_child_q.get(entity).unwrap();
        let Some(parent_entity) = entry.parent_id.get(&server) else {
            panic!("no parent entity!")
        };
        let parent_key = key_q.get(parent_entity).unwrap();
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

    // on FileDependency Insert Event
    for event in fs_dependency_events.read() {
        let user_key = event.user_key;
        let entity = event.entity;

        let component = fs_dependency_q.get(entity).unwrap();

        let file_entity = component.file_entity.get(&server).unwrap();
        let dependency_entity = component.dependency_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(file_entity).unwrap().clone();
        let dependency_key = key_q.get(dependency_entity).unwrap().clone();

        let project = git_manager.project_mut(&project_key).unwrap();
        project.file_add_dependency(&file_key, &dependency_key);

        let content_entity_data = ContentEntityData::new_dependency(dependency_key.clone());
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &entity,
            &content_entity_data,
        );
        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);

        git_manager.queue_client_open_dependency(&user_key, &project_key, &dependency_key);

        info!(
            "inserted FileDependency(file: `{:?}`, dependency: `{:?}`)",
            file_key.name(),
            dependency_key.name()
        );
    }
}

pub fn insert_vertex_component_events(
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut component_waitlist: ResMut<ComponentWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut vert_3d_events: EventReader<InsertComponentEvent<Vertex3d>>,
    mut vert_root_events: EventReader<InsertComponentEvent<VertexRoot>>,
) {
    // on Vertex3D Insert Event
    for event in vert_3d_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, inserted Vertex3d", entity);

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut None,
            &entity,
            ComponentWaitlistInsert::Vertex,
        );
    }

    // on VertexRoot Insert Event
    for event in vert_root_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, inserted VertexRoot", entity);

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut None,
            &entity,
            ComponentWaitlistInsert::VertexRoot,
        );
    }
}

pub fn insert_edge_component_events(
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut component_waitlist: ResMut<ComponentWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut edge_3d_events: EventReader<InsertComponentEvent<Edge3d>>,
    edge_3d_q: Query<&Edge3d>,
) {
    // on Edge3d Insert Event
    for event in edge_3d_events.read() {
        let edge_entity = event.entity;
        info!("entity: `{:?}`, inserted Edge3d", edge_entity);

        let edge_3d = edge_3d_q.get(edge_entity).unwrap();
        let Some(start_entity) = edge_3d.start.get(&server) else {
            panic!("no parent entity!")
        };
        let Some(end_entity) = edge_3d.end.get(&server) else {
            panic!("no child entity!")
        };
        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut None,
            &edge_entity,
            ComponentWaitlistInsert::Edge(start_entity, end_entity),
        );
    }
}

pub fn insert_face_component_events(
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut component_waitlist: ResMut<ComponentWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut face_3d_events: EventReader<InsertComponentEvent<Face3d>>,
    face_3d_q: Query<&Face3d>,
) {
    // on Face3d Insert Event
    for event in face_3d_events.read() {
        let entity = event.entity;
        let face_3d = face_3d_q.get(entity).unwrap();

        let vertex_a = face_3d.vertex_a.get(&server).unwrap();
        let vertex_b = face_3d.vertex_b.get(&server).unwrap();
        let vertex_c = face_3d.vertex_c.get(&server).unwrap();

        info!(
            "entity: `{:?}`, inserted Face3d(vertices({:?}, {:?}, {:?})))",
            entity, vertex_a, vertex_b, vertex_c
        );

        component_waitlist.process_inserts(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut None,
            &entity,
            &[
                ComponentWaitlistInsert::Face(
                    None, vertex_a, vertex_b, vertex_c
                ),
                ComponentWaitlistInsert::FileType(FileExtension::Mesh),
            ],
        );
    }
}

pub fn insert_shape_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut component_waitlist: ResMut<ComponentWaitlist>,
    mut shape_manager: ResMut<ShapeManager>,
    mut icon_manager: ResMut<IconManager>,
    mut file_type_events: EventReader<InsertComponentEvent<FileType>>,
    mut owned_by_file_events: EventReader<InsertComponentEvent<OwnedByFile>>,
    mut shape_name_events: EventReader<InsertComponentEvent<ShapeName>>,
    key_q: Query<&FileKey>,
    file_type_q: Query<&FileType>,
    owned_by_file_q: Query<&OwnedByFile>,
    shape_name_q: Query<&ShapeName>,
) {
    // on FileType Insert Event
    for event in file_type_events.read() {
        let entity = event.entity;

        let file_type_value = *file_type_q.get(entity).unwrap().value;

        info!(
            "entity: `{:?}`, inserted FileType: {:?}",
            entity, file_type_value
        );

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut None,
            &entity,
            ComponentWaitlistInsert::FileType(file_type_value),
        );
    }

    // on OwnedByFile Insert Event
    for event in owned_by_file_events.read() {
        let user_key = event.user_key;
        let entity = event.entity;
        let file_entity = owned_by_file_q
            .get(entity)
            .unwrap()
            .file_entity
            .get(&server)
            .unwrap();
        let file_key = key_q.get(file_entity).unwrap();
        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();

        info!(
            "entity: `{:?}`, inserted OwnedByFile({:?})",
            entity, file_entity
        );

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut Some(&mut icon_manager),
            &entity,
            ComponentWaitlistInsert::OwnedByFile(project_key, file_key.clone()),
        );
    }

    // on ShapeName Insert Event
    for event in shape_name_events.read() {
        let entity = event.entity;
        let shape_name = (*shape_name_q.get(entity).unwrap().value).clone();

        info!(
            "entity: `{:?}`, inserted ShapeName: {:?}",
            entity, shape_name
        );

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut Some(&mut shape_manager),
            &mut None,
            &entity,
            ComponentWaitlistInsert::ShapeName,
        );

        //

        if let Ok(owned_by_file) = owned_by_file_q.get(entity) {
            // entity is a SkelVertex?

            let user_key = event.user_key;
            let file_entity = owned_by_file.file_entity.get(&server).unwrap();

            let project_key = user_manager
                .user_session_data(&user_key)
                .unwrap()
                .project_key()
                .unwrap();
            let file_key = key_q.get(file_entity).unwrap().clone();

            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
    }
}

pub fn insert_icon_component_events(
    mut server: Server,
    mut commands: Commands,

    mut vertex_events: EventReader<InsertComponentEvent<IconVertex>>,
    mut edge_events: EventReader<InsertComponentEvent<IconEdge>>,
    mut face_events: EventReader<InsertComponentEvent<IconFace>>,
    mut frame_events: EventReader<InsertComponentEvent<IconFrame>>,

    mut git_manager: ResMut<GitManager>,
    user_manager: Res<UserManager>,
    mut component_waitlist: ResMut<ComponentWaitlist>,
    mut icon_manager: ResMut<IconManager>,

    key_q: Query<&FileKey>,
    edge_q: Query<&IconEdge>,
    face_q: Query<&IconFace>,
    mut frame_q: Query<&mut IconFrame>,
) {
    // on IconVertex Insert Event
    for event in vertex_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, inserted IconVertex", entity);

        component_waitlist.process_inserts(
            &mut server,
            &mut git_manager,
            &mut None,
            &mut Some(&mut icon_manager),
            &entity,
            &[
                ComponentWaitlistInsert::Vertex,
                ComponentWaitlistInsert::FileType(FileExtension::Icon),
            ],
        );
    }

    // on IconEdge Insert Event
    for event in edge_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, inserted IconEdge", entity);

        let edge = edge_q.get(entity).unwrap();
        let Some(start_entity) = edge.start.get(&server) else {
            panic!("no parent entity!")
        };
        let Some(end_entity) = edge.end.get(&server) else {
            panic!("no child entity!")
        };
        component_waitlist.process_inserts(
            &mut server,
            &mut git_manager,
            &mut None,
            &mut Some(&mut icon_manager),
            &entity,
            &[
                ComponentWaitlistInsert::Edge(start_entity, end_entity),
                ComponentWaitlistInsert::FileType(FileExtension::Icon),
            ],
        );
    }

    // on IconFace Insert Event
    for event in face_events.read() {
        let entity = event.entity;
        let face = face_q.get(entity).unwrap();

        let frame_entity = face.frame_entity.get(&server).unwrap();
        let vertex_a = face.vertex_a.get(&server).unwrap();
        let vertex_b = face.vertex_b.get(&server).unwrap();
        let vertex_c = face.vertex_c.get(&server).unwrap();

        info!(
            "entity: `{:?}`, inserted IconFace(vertices({:?}, {:?}, {:?})",
            entity, vertex_a, vertex_b, vertex_c
        );

        component_waitlist.process_inserts(
            &mut server,
            &mut git_manager,
            &mut None,
            &mut Some(&mut icon_manager),
            &entity,
            &[
                ComponentWaitlistInsert::Face(
                    Some(frame_entity),
                    vertex_a,
                    vertex_b,
                    vertex_c,
                ),
                ComponentWaitlistInsert::FileType(FileExtension::Icon),
            ],
        );
    }

    // on IconFrame Insert Event
    for event in frame_events.read() {
        let user_key = event.user_key;
        let frame_entity = event.entity;
        info!("entity: `{:?}`, inserted IconFrame", frame_entity);

        let frame = frame_q.get(frame_entity).unwrap();
        let frame_index = frame.get_order() as usize;
        let file_entity: Entity = frame.file_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(file_entity).unwrap().clone();

        icon_manager.on_create_frame(&file_entity, &frame_entity, frame_index, Some(&mut frame_q));

        let content_entity_data = ContentEntityData::new_frame();
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &frame_entity,
            &content_entity_data,
        );

        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
}

pub fn insert_animation_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: ResMut<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut animation_manager: ResMut<AnimationManager>,
    mut frame_events: EventReader<InsertComponentEvent<AnimFrame>>,
    mut rotation_events: EventReader<InsertComponentEvent<AnimRotation>>,
    key_q: Query<&FileKey>,
    mut frame_q: Query<&mut AnimFrame>,
    rot_q: Query<&AnimRotation>,
) {
    // on AnimFrame Insert Event
    for event in frame_events.read() {
        let user_key = event.user_key;
        let frame_entity = event.entity;
        info!("entity: `{:?}`, inserted AnimFrame", frame_entity);

        let frame = frame_q.get(frame_entity).unwrap();
        let frame_index = frame.get_order() as usize;
        let file_entity: Entity = frame.file_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(file_entity).unwrap().clone();

        animation_manager.on_create_frame(
            &file_entity,
            &frame_entity,
            frame_index,
            Some(&mut frame_q),
        );

        let content_entity_data = ContentEntityData::new_frame();
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &frame_entity,
            &content_entity_data,
        );

        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }

    // on AnimRotation Insert Event
    for event in rotation_events.read() {
        let user_key = event.user_key;
        let rot_entity = event.entity;

        info!("entity: `{:?}`, inserted AnimRotation", rot_entity);

        let rotation = rot_q.get(rot_entity).unwrap();
        let frame_entity: Entity = rotation.frame_entity.get(&server).unwrap();
        let frame = frame_q.get(frame_entity).unwrap();
        let file_entity: Entity = frame.file_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(file_entity).unwrap().clone();

        animation_manager.on_create_rotation(frame_entity, rot_entity);

        let content_entity_data = ContentEntityData::new_rotation();
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &rot_entity,
            &content_entity_data,
        );

        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
}

pub fn insert_palette_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: ResMut<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut palette_manager: ResMut<PaletteManager>,
    mut color_events: EventReader<InsertComponentEvent<PaletteColor>>,
    key_q: Query<&FileKey>,
    mut color_q: Query<&mut PaletteColor>,
) {
    // on PaletteColor Insert Event
    for event in color_events.read() {
        let user_key = event.user_key;
        let color_entity = event.entity;
        info!("entity: `{:?}`, inserted PaletteColor", color_entity);

        let color = color_q.get(color_entity).unwrap();
        let color_index = *color.index as usize;
        let file_entity: Entity = color.owning_file_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(file_entity).unwrap().clone();

        palette_manager.on_create_color(
            &file_entity,
            &color_entity,
            color_index,
            Some(&mut color_q),
        );

        let content_entity_data = ContentEntityData::new_palette_color();
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &color_entity,
            &content_entity_data,
        );

        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
}

pub fn insert_skin_component_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: ResMut<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut skin_manager: ResMut<SkinManager>,
    mut bckg_events: EventReader<InsertComponentEvent<BackgroundSkinColor>>,
    mut color_events: EventReader<InsertComponentEvent<FaceColor>>,
    key_q: Query<&FileKey>,
    bckg_q: Query<&BackgroundSkinColor>,
    color_q: Query<&FaceColor>,
) {
    // on BackgroundSkinColor Insert Event
    for event in bckg_events.read() {
        let user_key = event.user_key;
        let color_entity = event.entity;
        info!("entity: `{:?}`, inserted BackgroundSkinColor", color_entity);

        let color = bckg_q.get(color_entity).unwrap();
        let skin_file_entity: Entity = color.owning_file_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(skin_file_entity).unwrap().clone();

        let content_entity_data = ContentEntityData::new_background_color(None);
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &color_entity,
            &content_entity_data,
        );

        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
    // on FaceColor Insert Event
    for event in color_events.read() {
        let user_key = event.user_key;
        let color_entity = event.entity;
        info!("entity: `{:?}`, inserted FaceColor", color_entity);

        let color = color_q.get(color_entity).unwrap();
        let face_3d_entity: Entity = color.face_entity.get(&server).unwrap();
        let skin_file_entity: Entity = color.owning_file_entity.get(&server).unwrap();

        let project_key = user_manager
            .user_session_data(&user_key)
            .unwrap()
            .project_key()
            .unwrap();
        let file_key = key_q.get(skin_file_entity).unwrap().clone();

        skin_manager.on_create_face_color(&face_3d_entity, &color_entity);

        let content_entity_data = ContentEntityData::new_skin_color(None);
        git_manager.on_insert_content_entity(
            &mut server,
            &project_key,
            &file_key,
            &color_entity,
            &content_entity_data,
        );

        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
}

pub fn insert_model_component_events(
    mut transform_events: EventReader<InsertComponentEvent<NetTransform>>,
    mut skin_or_scene_events: EventReader<InsertComponentEvent<SkinOrSceneEntity>>,
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    mut component_waitlist: ResMut<ComponentWaitlist>,
) {
    // on NetTransform Insert Event
    for event in transform_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, inserted NetTransform", entity,);

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut None,
            &mut None,
            &entity,
            ComponentWaitlistInsert::NetTransform,
        );
    }

    // on SkinOrSceneEntity Insert Event
    for event in skin_or_scene_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, inserted SkinOrSceneEntity", entity,);

        component_waitlist.process_insert(
            &mut server,
            &mut git_manager,
            &mut None,
            &mut None,
            &entity,
            ComponentWaitlistInsert::SkinOrSceneEntity,
        );
    }
}
