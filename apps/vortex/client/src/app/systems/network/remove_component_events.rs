use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, ResMut},
};
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{Resource, SystemState};
use bevy_ecs::world::{Mut, World};
use bevy_log::info;

use naia_bevy_client::{events::RemoveComponentEvents, Client, Replicate};

use render_api::{base::CpuMesh, Assets};

use vortex_proto::components::{AnimFrame, AnimRotation, BackgroundSkinColor, ChangelistEntry, ChangelistStatus, Edge3d, Face3d, FaceColor, FileDependency, FileSystemChild, FileSystemEntry, FileSystemRootChild, ModelTransform, PaletteColor, ShapeName, Vertex3d};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    resources::{
        animation_manager::AnimationManager, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, file_manager::FileManager, input::InputManager,
        palette_manager::PaletteManager, skin_manager::SkinManager, tab_manager::TabManager,
        vertex_manager::VertexManager, model_manager::ModelManager,
    },
};
use crate::app::events::RemoveComponentEvent;


#[derive(Resource)]
struct CachedRemoveComponentEventsState {
    event_state: SystemState<EventReader<'static, 'static, RemoveComponentEvents>>,
}

pub fn remove_component_event_startup(world: &mut World) {
    let initial_state: SystemState<EventReader<RemoveComponentEvents>> = SystemState::new(world);
    world.insert_resource(CachedRemoveComponentEventsState {
        event_state: initial_state,
    });
}

pub fn remove_component_events(world: &mut World) {
    let mut events_collection: Vec<RemoveComponentEvents> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedRemoveComponentEventsState>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.iter() {
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
        remove_component_event::<AnimFrame>(world, &events);
        remove_component_event::<AnimRotation>(world, &events);
        remove_component_event::<PaletteColor>(world, &events);
        remove_component_event::<FaceColor>(world, &events);
        remove_component_event::<BackgroundSkinColor>(world, &events);
        remove_component_event::<ModelTransform>(world, &events);
    }
}

fn remove_component_event<T: Replicate>(world: &mut World, events: &RemoveComponentEvents) {
    let mut system_state: SystemState<EventWriter<RemoveComponentEvent<T>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for (entity, component) in events.read::<T>() {
        event_writer.send(RemoveComponentEvent::<T>::new(entity, component));
    }
}

todo!(convert this to separate systems!);
pub fn remove_component_events_old(
    mut commands: Commands,
    mut client: Client,
    mut canvas: ResMut<Canvas>,
    mut file_manager: ResMut<FileManager>,
    mut input_manager: ResMut<InputManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut tab_manager: ResMut<TabManager>,
    mut animation_manager: ResMut<AnimationManager>,
    mut palette_manager: ResMut<PaletteManager>,
    mut skin_manager: ResMut<SkinManager>,
    mut model_manager: ResMut<ModelManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut event_reader: EventReader<RemoveComponentEvents>,
    mut parent_q: Query<&mut FileSystemParent>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    for events in event_reader.iter() {
        for (entity, _component) in events.read::<FileSystemEntry>() {
            info!("entity: `{:?}`, removed FileSystemEntry", entity);

            file_manager.on_file_delete(&mut client, &mut tab_manager, &entity);
        }

        for (entity, _component) in events.read::<FileSystemRootChild>() {
            info!("entity: `{:?}`, removed FileSystemRootChild", entity);

            let Ok(mut parent) = parent_q.get_mut(file_manager.project_root_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }
        for (entity, component) in events.read::<FileSystemChild>() {
            info!("entity: `{:?}`, removed FileSystemChild", entity);

            let Some(parent_entity) = component.parent_id.get(&client) else {
                continue;
            };
            let Ok(mut parent) = parent_q.get_mut(parent_entity) else {
                continue;
            };
            parent.remove_child(&entity);
        }
        for (entity, component) in events.read::<ChangelistEntry>() {
            info!("entity: `{:?}`, removed ChangelistEntry", entity);

            let entry = component.file_key();
            file_manager.remove_changelist_entry(&entry);

            if *component.status != ChangelistStatus::Deleted {
                if let Some(file_entity) = component.file_entity.get(&client) {
                    if let Ok(mut fs_state) = fs_state_q.get_mut(file_entity) {
                        fs_state.change_status = None;
                    }
                }
            }
        }
        for (entity, component) in events.read::<FileDependency>() {
            info!("entity: `{:?}`, removed FileDependency", entity);

            let file_entity = component.file_entity.get(&client).unwrap();
            let dependency_entity = component.dependency_entity.get(&client).unwrap();

            file_manager.file_remove_dependency(&file_entity, &dependency_entity);
        }
        for (vertex_entity_3d, shape_name) in events.read::<ShapeName>() {
            let name = (*shape_name.value).clone();

            info!(
                "entity: `{:?}`, removed ShapeName(name: {:?})",
                vertex_entity_3d, name
            );
        }
        for (vertex_entity_3d, _) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, removed Vertex3d", vertex_entity_3d);

            vertex_manager.cleanup_deleted_vertex(
                &mut commands,
                &mut canvas,
                &mut input_manager,
                &vertex_entity_3d,
            );
        }
        for (edge_3d_entity, _) in events.read::<Edge3d>() {
            info!("entity: `{:?}`, removed Edge3d", edge_3d_entity);

            edge_manager.cleanup_deleted_edge(
                &mut commands,
                &mut canvas,
                &mut input_manager,
                &mut vertex_manager,
                &mut face_manager,
                &edge_3d_entity,
            );
        }
        for (face_entity_3d, _) in events.read::<Face3d>() {
            info!("entity: `{:?}`, removed Face3d", face_entity_3d);

            face_manager.cleanup_deleted_face_3d(&mut commands, &mut meshes, &face_entity_3d);
        }
        for (frame_entity, frame) in events.read::<AnimFrame>() {
            info!("entity: `{:?}`, removed AnimFrame", frame_entity);

            let file_entity = frame.file_entity.get(&client).unwrap();

            animation_manager.deregister_frame(&file_entity, &frame_entity);
        }
        for (rot_entity, _) in events.read::<AnimRotation>() {
            info!("entity: `{:?}`, removed AnimRotation", rot_entity);

            animation_manager.deregister_rotation(&rot_entity);
        }
        for (color_entity, color) in events.read::<PaletteColor>() {
            info!("entity: `{:?}`, removed PaletteColor", color_entity);

            let file_entity = color.file_entity.get(&client).unwrap();
            let color_index = *color.index as usize;

            palette_manager.deregister_color(&file_entity, &color_entity, color_index);
        }
        for (color_entity, _color) in events.read::<BackgroundSkinColor>() {
            info!("entity: `{:?}`, removed BackgroundSkinColor", color_entity);

            skin_manager.deregister_bckg_color(&color_entity);
        }
        for (color_entity, _color) in events.read::<FaceColor>() {
            info!("entity: `{:?}`, removed FaceColor", color_entity);

            skin_manager.deregister_face_color(&color_entity);
        }
        for (entity, _transform) in events.read::<ModelTransform>() {
            info!("entity: `{:?}`, removed ModelTransform", entity);

            model_manager.on_despawn_model_transform(&mut commands, &entity);
        }
    }
}
