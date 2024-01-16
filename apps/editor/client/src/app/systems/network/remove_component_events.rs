use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Commands, Query, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::info;

use naia_bevy_client::{events::RemoveComponentEvents, Client, Replicate};

use render_api::{base::CpuMesh, Assets};

use editor_proto::components::{
    AnimFrame, AnimRotation, BackgroundSkinColor, ChangelistEntry, ChangelistStatus, Edge3d,
    Face3d, FaceColor, FileDependency, FileSystemChild, FileSystemEntry, FileSystemRootChild,
    IconEdge, IconFace, IconFrame, IconVertex, NetTransform, PaletteColor, ShapeName, Vertex3d,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    events::RemoveComponentEvent,
    resources::{
        animation_manager::AnimationManager, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, file_manager::FileManager, icon_manager::IconManager,
        input::InputManager, model_manager::ModelManager, palette_manager::PaletteManager,
        skin_manager::SkinManager, tab_manager::TabManager, vertex_manager::VertexManager,
    },
    plugin::Main,
};

#[derive(Resource)]
struct CachedRemoveComponentEventsState {
    event_state: SystemState<EventReader<'static, 'static, RemoveComponentEvents<Main>>>,
}

pub fn remove_component_event_startup(world: &mut World) {
    let initial_state: SystemState<EventReader<RemoveComponentEvents<Main>>> = SystemState::new(world);
    world.insert_resource(CachedRemoveComponentEventsState {
        event_state: initial_state,
    });
}

pub fn remove_component_events(world: &mut World) {
    let mut events_collection: Vec<RemoveComponentEvents<Main>> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedRemoveComponentEventsState>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.read() {
                events_collection.push(events.clone_new());
            }
        },
    );

    for events in events_collection {
        remove_component_event::<FileSystemEntry>(world, &events);
        remove_component_event::<FileSystemRootChild>(world, &events);
        remove_component_event::<FileSystemChild>(world, &events);
        remove_component_event::<FileDependency>(world, &events);
        remove_component_event::<ChangelistEntry>(world, &events);
        remove_component_event::<Vertex3d>(world, &events);
        remove_component_event::<Edge3d>(world, &events);
        remove_component_event::<Face3d>(world, &events);
        remove_component_event::<IconVertex>(world, &events);
        remove_component_event::<IconEdge>(world, &events);
        remove_component_event::<IconFace>(world, &events);
        remove_component_event::<IconFrame>(world, &events);
        remove_component_event::<AnimFrame>(world, &events);
        remove_component_event::<AnimRotation>(world, &events);
        remove_component_event::<PaletteColor>(world, &events);
        remove_component_event::<FaceColor>(world, &events);
        remove_component_event::<BackgroundSkinColor>(world, &events);
        remove_component_event::<NetTransform>(world, &events);
    }
}

fn remove_component_event<T: Replicate>(world: &mut World, events: &RemoveComponentEvents<Main>) {
    let mut system_state: SystemState<EventWriter<RemoveComponentEvent<T>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for (entity, component) in events.read::<T>() {
        event_writer.send(RemoveComponentEvent::<T>::new(entity, component));
    }
}

pub fn remove_file_component_events(
    mut entry_events: EventReader<RemoveComponentEvent<FileSystemEntry>>,
    mut root_child_events: EventReader<RemoveComponentEvent<FileSystemRootChild>>,
    mut child_events: EventReader<RemoveComponentEvent<FileSystemChild>>,
    mut cl_events: EventReader<RemoveComponentEvent<ChangelistEntry>>,
    mut dependencies_events: EventReader<RemoveComponentEvent<FileDependency>>,
    mut client: Client<Main>,
    mut file_manager: ResMut<FileManager>,
    mut tab_manager: ResMut<TabManager>,
    mut parent_q: Query<&mut FileSystemParent>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    for event in entry_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed FileSystemEntry", entity);

        file_manager.on_file_delete(&mut client, &mut tab_manager, &entity);
    }

    for event in root_child_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed FileSystemRootChild", entity);

        let Ok(mut parent) = parent_q.get_mut(file_manager.project_root_entity) else {
            continue;
        };
        parent.remove_child(&entity);
    }

    for event in child_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed FileSystemChild", entity);

        let Some(parent_entity) = event.component.parent_id.get(&client) else {
            continue;
        };
        let Ok(mut parent) = parent_q.get_mut(parent_entity) else {
            continue;
        };
        parent.remove_child(&entity);
    }

    for event in cl_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed ChangelistEntry", entity);

        let entry = event.component.file_key();
        file_manager.remove_changelist_entry(&entry);

        if *event.component.status != ChangelistStatus::Deleted {
            if let Some(file_entity) = event.component.file_entity.get(&client) {
                if let Ok(mut fs_state) = fs_state_q.get_mut(file_entity) {
                    fs_state.change_status = None;
                }
            }
        }
    }
    for event in dependencies_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed FileDependency", entity);

        let file_entity = event.component.file_entity.get(&client).unwrap();
        let dependency_entity = event.component.dependency_entity.get(&client).unwrap();

        file_manager.file_remove_dependency(&file_entity, &dependency_entity);
    }
}

pub fn remove_shape_component_events(
    mut shape_name_events: EventReader<RemoveComponentEvent<ShapeName>>,
    mut vertex_events: EventReader<RemoveComponentEvent<Vertex3d>>,
    mut edge_events: EventReader<RemoveComponentEvent<Edge3d>>,
    mut face_events: EventReader<RemoveComponentEvent<Face3d>>,
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    mut input_manager: ResMut<InputManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
) {
    for event in shape_name_events.read() {
        let entity = event.entity;

        let name = (*event.component.value).clone();

        info!(
            "entity: `{:?}`, removed ShapeName(name: {:?})",
            entity, name
        );
    }

    for event in vertex_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed Vertex3d", entity);

        vertex_manager.cleanup_deleted_vertex(
            &mut commands,
            &mut canvas,
            &mut input_manager,
            &entity,
        );
    }
    for event in edge_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed Edge3d", entity);

        edge_manager.cleanup_deleted_edge(
            &mut commands,
            &mut canvas,
            &mut input_manager,
            &mut vertex_manager,
            Some(&mut face_manager),
            &entity,
        );
    }
    for event in face_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed Face3d", entity);

        face_manager.cleanup_deleted_face_3d(&mut commands, &mut meshes, &entity);
    }
}

pub fn remove_icon_component_events(
    client: Client<Main>,

    mut vertex_events: EventReader<RemoveComponentEvent<IconVertex>>,
    mut edge_events: EventReader<RemoveComponentEvent<IconEdge>>,
    mut face_events: EventReader<RemoveComponentEvent<IconFace>>,
    mut frame_events: EventReader<RemoveComponentEvent<IconFrame>>,

    mut commands: Commands,
    mut icon_manager: ResMut<IconManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
) {
    for event in vertex_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed IconVertex", entity);

        icon_manager.cleanup_deleted_vertex(&entity);
    }
    for event in edge_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed IconEdge", entity);

        icon_manager.cleanup_deleted_edge(&mut commands, &entity);
    }
    for event in face_events.read() {
        let entity = event.entity;

        info!("entity: `{:?}`, removed IconFace", entity);

        icon_manager.cleanup_deleted_net_face(&mut commands, &mut meshes, &entity);
    }
    for event in frame_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed IconFrame", entity);

        let file_entity = event.component.file_entity.get(&client).unwrap();

        icon_manager.deregister_frame(&file_entity, &entity);
    }
}

pub fn remove_animation_component_events(
    mut anim_frame_events: EventReader<RemoveComponentEvent<AnimFrame>>,
    mut anim_rotation_events: EventReader<RemoveComponentEvent<AnimRotation>>,
    client: Client<Main>,
    mut animation_manager: ResMut<AnimationManager>,
) {
    for event in anim_frame_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed AnimFrame", entity);

        let file_entity = event.component.file_entity.get(&client).unwrap();

        animation_manager.deregister_frame(&file_entity, &entity);
    }
    for event in anim_rotation_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed AnimRotation", entity);

        animation_manager.deregister_rotation(&entity);
    }
}

pub fn remove_color_component_events(
    mut palette_events: EventReader<RemoveComponentEvent<PaletteColor>>,
    mut bck_color_events: EventReader<RemoveComponentEvent<BackgroundSkinColor>>,
    mut face_color_events: EventReader<RemoveComponentEvent<FaceColor>>,
    client: Client<Main>,
    mut palette_manager: ResMut<PaletteManager>,
    mut skin_manager: ResMut<SkinManager>,
) {
    for event in palette_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed PaletteColor", entity);

        let file_entity = event.component.owning_file_entity.get(&client).unwrap();
        let color_index = *event.component.index as usize;

        palette_manager.deregister_color(&file_entity, &entity, color_index);
    }

    for event in bck_color_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed BackgroundSkinColor", entity);

        skin_manager.deregister_bckg_color(&entity);
    }

    for event in face_color_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed FaceColor", entity);

        skin_manager.deregister_face_color(&entity);
    }
}

pub fn remove_model_component_events(
    mut net_transform_events: EventReader<RemoveComponentEvent<NetTransform>>,
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    mut input_manager: ResMut<InputManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut model_manager: ResMut<ModelManager>,
) {
    for event in net_transform_events.read() {
        let entity = event.entity;
        info!("entity: `{:?}`, removed NetTransform", entity);

        model_manager.on_despawn_net_transform(
            &mut commands,
            &mut canvas,
            &mut input_manager,
            &mut vertex_manager,
            &mut edge_manager,
            &entity,
        );
    }
}
