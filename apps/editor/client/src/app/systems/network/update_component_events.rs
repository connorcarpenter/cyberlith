use bevy_ecs::{
    event::EventReader,
    prelude::EventWriter,
    system::{Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::info;

use naia_bevy_client::{events::UpdateComponentEvents, Client};

use render_api::{base::CpuMesh, components::Transform, Assets, Handle};

use editor_proto::components::{
    AnimFrame, AnimRotation, BackgroundSkinColor, ChangelistEntry, EdgeAngle, Face3d, FaceColor,
    FileSystemChild, FileSystemEntry, FileSystemRootChild, IconFrame, PaletteColor, ShapeName,
    Vertex3d,
};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemEntryLocal},
    events::ShapeColorResyncEvent,
    plugin::Main,
    resources::{
        animation_manager::AnimationManager,
        canvas::Canvas,
        face_manager::FaceManager,
        file_manager::{get_full_path, FileManager},
        icon_manager::IconManager,
        palette_manager::PaletteManager,
        vertex_manager::VertexManager,
    },
};

#[derive(Resource)]
struct CachedUpdateComponentEventsState {
    event_state: SystemState<EventReader<'static, 'static, UpdateComponentEvents<Main>>>,
}

pub fn update_component_event_startup(world: &mut World) {
    let initial_state: SystemState<EventReader<UpdateComponentEvents<Main>>> =
        SystemState::new(world);
    world.insert_resource(CachedUpdateComponentEventsState {
        event_state: initial_state,
    });
}

pub fn update_component_events(world: &mut World) {
    let mut events_collection: Vec<UpdateComponentEvents<Main>> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedUpdateComponentEventsState>| {
            let mut event_reader = events_reader_state.event_state.get_mut(world);

            for events in event_reader.read() {
                events_collection.push(events.clone());
            }
        },
    );
    for events in events_collection {
        let mut system_state: SystemState<(
            Client<Main>,
            ResMut<FileManager>,
            Query<(&FileSystemEntry, Option<&FileSystemChild>)>,
            Query<&mut FileSystemEntryLocal>,
            Query<(&ChangelistEntry, &mut ChangelistUiState)>,
        )> = SystemState::new(world);
        let (client, file_manager, entry_q, mut entry_local_q, mut cl_q) =
            system_state.get_mut(world);

        // on FileSystemEntry Update Event
        for (_, entry_entity) in events.read::<FileSystemEntry>() {
            let (entry, _) = entry_q.get(entry_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemEntry: `{:?}` ({:?})",
                entry_entity, entry_name
            );
            if let Some(cl_entity) = file_manager.get_file_changelist_entity(&entry_entity) {
                let (_, mut cl_state) = cl_q.get_mut(cl_entity).unwrap();
                cl_state.display_name = entry_name.clone();
            }
            if let Some(cl_children) = file_manager.get_file_changelist_children(&entry_entity) {
                for cl_child_entity in cl_children.iter() {
                    let (cl_entry, old_child_state) = cl_q.get(*cl_child_entity).unwrap();
                    let child_file_entity = cl_entry.file_entity.get(&client).unwrap();

                    let old_path = old_child_state.display_path.clone();
                    let new_path = get_full_path(&client, &entry_q, child_file_entity);
                    info!(
                        "change path for child entity: `{:?}`. path was: `{:?}`, now is `{:?}`",
                        cl_child_entity, old_path, new_path
                    );

                    let (_, mut cl_child_state) = cl_q.get_mut(*cl_child_entity).unwrap();
                    cl_child_state.display_path = new_path;
                }
            }
            let mut entry_local = entry_local_q.get_mut(entry_entity).unwrap();
            entry_local.name = entry_name;
        }
        // on FileSystemRootChild Update Event
        for (_, child_entity) in events.read::<FileSystemRootChild>() {
            let (entry, _) = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemRootChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }
        // on FileSystemChild Update Event
        for (_, child_entity) in events.read::<FileSystemChild>() {
            let (entry, _) = entry_q.get(child_entity).unwrap();
            let entry_name = (*(entry.name)).clone();
            info!(
                "received updated FileSystemChild: `{:?}` ({:?})",
                child_entity, entry_name
            );
            todo!();
        }

        let mut system_state: SystemState<(
            Client<Main>,
            ResMut<Canvas>,
            Res<VertexManager>,
            Res<FaceManager>,
            ResMut<IconManager>,
            ResMut<AnimationManager>,
            ResMut<PaletteManager>,
            ResMut<Assets<CpuMesh>>,
            Query<&AnimFrame>,
            Query<&IconFrame>,
            Query<&PaletteColor>,
            Query<&Handle<CpuMesh>>,
            Query<&Face3d>,
            Query<&mut Transform>,
            EventWriter<ShapeColorResyncEvent>,
        )> = SystemState::new(world);
        let (
            client,
            mut canvas,
            vertex_manager,
            face_manager,
            mut icon_manager,
            mut animation_manager,
            mut palette_manager,
            mut meshes,
            anim_frame_q,
            icon_frame_q,
            color_q,
            mesh_handle_q,
            face_3d_q,
            mut transform_q,
            mut shape_color_resync_events,
        ) = system_state.get_mut(world);

        // on Shape Update Event
        let mut updated_shapes = false;
        for (_, vertex_3d_entity) in events.read::<Vertex3d>() {
            updated_shapes = true;
            vertex_manager.on_vertex_3d_moved(
                &client,
                &face_manager,
                &mut meshes,
                &mesh_handle_q,
                &face_3d_q,
                &mut transform_q,
                &vertex_3d_entity,
            );
        }

        for (_, _) in events.read::<EdgeAngle>() {
            updated_shapes = true;
        }
        for (_, _) in events.read::<AnimRotation>() {
            updated_shapes = true;
        }
        if updated_shapes {
            canvas.queue_resync_shapes();
        }
        for (_tick, _entity) in events.read::<ShapeName>() {
            // what to do here? update some caches?
        }
        for (_tick, frame_entity) in events.read::<AnimFrame>() {
            let Ok(frame) = anim_frame_q.get(frame_entity) else {
                panic!("AnimFrame component not found for entity `{:?}`", frame_entity);
            };
            let file_entity = frame.file_entity.get(&client).unwrap();
            // check that index has changed
            let frame_index = frame.get_order() as usize;
            let existing_frame_entity =
                animation_manager.get_frame_entity(&file_entity, frame_index);
            if existing_frame_entity != Some(frame_entity) {
                animation_manager.framing_queue_resync_frame_order(&file_entity);
            }
        }
        for (_tick, frame_entity) in events.read::<IconFrame>() {
            let Ok(frame) = icon_frame_q.get(frame_entity) else {
                panic!("IconFrame component not found for entity `{:?}`", frame_entity);
            };
            let file_entity = frame.file_entity.get(&client).unwrap();
            // check that index has changed
            let frame_index = frame.get_order() as usize;
            let existing_frame_entity = icon_manager.get_frame_entity(&file_entity, frame_index);
            if existing_frame_entity != Some(frame_entity) {
                icon_manager.framing_queue_resync_frame_order(&file_entity);
            }
        }
        for (_tick, color_entity) in events.read::<PaletteColor>() {
            let Ok(color) = color_q.get(color_entity) else {
                panic!("color component not found for entity `{:?}`", color_entity);
            };
            let file_entity = color.owning_file_entity.get(&client).unwrap();
            // check that index has changed
            let color_index = *color.index as usize;
            let existing_color_entity = palette_manager.get_color_entity(&file_entity, color_index);
            if existing_color_entity != Some(color_entity) {
                palette_manager.queue_resync_color_order(&file_entity);
            }
        }
        let mut updated_colors = false;
        for (_tick, _entity) in events.read::<BackgroundSkinColor>() {
            updated_colors = true;
        }
        for (_tick, _entity) in events.read::<FaceColor>() {
            updated_colors = true;
        }

        if updated_colors {
            shape_color_resync_events.send(ShapeColorResyncEvent);
        }
    }
}
