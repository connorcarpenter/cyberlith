use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use render_api::{resources::WindowSettings, Draw};

use editor_proto::{
    components::{
        AnimFrame, AnimRotation, BackgroundSkinColor, ChangelistEntry, Edge3d, EdgeAngle,
        EntryKind, Face3d, FaceColor, FileDependency, FileSystemChild, FileSystemEntry,
        FileSystemRootChild, FileType, IconEdge, IconFace, IconFrame, IconVertex, NetTransform,
        OwnedByFile, PaletteColor, ShapeName, SkinOrSceneEntity, Vertex3d, VertexRoot,
    },
    protocol,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    config::ConfigPlugin,
    events::{InsertComponentEvent, LoginEvent, RemoveComponentEvent, ShapeColorResyncEvent},
    resources::{
        action::file::FileActions, animation_manager::AnimationManager,
        camera_manager::CameraManager, canvas::Canvas, compass::Compass,
        component_waitlist::ComponentWaitlist, edge_manager::EdgeManager,
        face_manager::FaceManager, file_manager::FileManager, grid::Grid,
        icon_manager::IconManager, input::InputManager, model_manager::ModelManager,
        palette_manager::PaletteManager, skin_manager::SkinManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
    systems::{canvas, draw, draw_vertices_and_edges, network, ui},
    ui::{
        widgets::{FrameInspectBarState, NamingBarState},
        UiState,
    },
};

#[derive(Clone)]
pub struct Main;

pub struct VortexPlugin;

impl Plugin for VortexPlugin {
    fn build(&self, app: &mut App) {
        // setup FileManager
        let project_root_entity = app
            .world_mut()
            .spawn_empty()
            .insert(FileSystemParent::new())
            .insert(FileSystemUiState::new_root())
            .insert(FileSystemEntry::new("Project", EntryKind::Directory))
            .id();
        let file_manager = FileManager::new(project_root_entity);

        app
            // Add Config
            .add_plugins(ConfigPlugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Vortex".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            // Networking Plugin
            .add_plugins(ClientPlugin::<Main>::new(
                ClientConfig::default(),
                protocol(),
            ))
            .add_event::<LoginEvent>()
            // Networking Systems
            .add_systems(Update, network::login)
            .add_systems(Startup, network::insert_component_event_startup)
            .add_systems(Startup, network::remove_component_event_startup)
            .add_systems(Startup, network::update_component_event_startup)
            .add_systems(Startup, network::auth_event_startup)
            .add_systems(
                Update,
                (
                    network::connect_events,
                    network::disconnect_events,
                    network::reject_events,
                    network::error_events,
                    network::spawn_entity_events,
                    network::despawn_entity_events,
                    network::insert_component_events,
                    network::update_component_events,
                    network::remove_component_events,
                    network::auth_events,
                )
                    .in_set(ReceiveEvents),
            )
            // Insert Component Events
            .add_event::<InsertComponentEvent<FileSystemEntry>>()
            .add_event::<InsertComponentEvent<FileSystemRootChild>>()
            .add_event::<InsertComponentEvent<FileSystemChild>>()
            .add_event::<InsertComponentEvent<ChangelistEntry>>()
            .add_event::<InsertComponentEvent<Vertex3d>>()
            .add_event::<InsertComponentEvent<VertexRoot>>()
            .add_event::<InsertComponentEvent<Edge3d>>()
            .add_event::<InsertComponentEvent<EdgeAngle>>()
            .add_event::<InsertComponentEvent<Face3d>>()
            .add_event::<InsertComponentEvent<IconVertex>>()
            .add_event::<InsertComponentEvent<IconEdge>>()
            .add_event::<InsertComponentEvent<IconFace>>()
            .add_event::<InsertComponentEvent<IconFrame>>()
            .add_event::<InsertComponentEvent<FileType>>()
            .add_event::<InsertComponentEvent<OwnedByFile>>()
            .add_event::<InsertComponentEvent<ShapeName>>()
            .add_event::<InsertComponentEvent<FileDependency>>()
            .add_event::<InsertComponentEvent<AnimFrame>>()
            .add_event::<InsertComponentEvent<AnimRotation>>()
            .add_event::<InsertComponentEvent<PaletteColor>>()
            .add_event::<InsertComponentEvent<BackgroundSkinColor>>()
            .add_event::<InsertComponentEvent<FaceColor>>()
            .add_event::<InsertComponentEvent<NetTransform>>()
            .add_event::<InsertComponentEvent<SkinOrSceneEntity>>()
            // Remove Component Events
            .add_event::<RemoveComponentEvent<FileSystemEntry>>()
            .add_event::<RemoveComponentEvent<FileSystemRootChild>>()
            .add_event::<RemoveComponentEvent<FileSystemChild>>()
            .add_event::<RemoveComponentEvent<ChangelistEntry>>()
            .add_event::<RemoveComponentEvent<FileDependency>>()
            .add_event::<RemoveComponentEvent<Vertex3d>>()
            .add_event::<RemoveComponentEvent<VertexRoot>>()
            .add_event::<RemoveComponentEvent<Edge3d>>()
            .add_event::<RemoveComponentEvent<Face3d>>()
            .add_event::<RemoveComponentEvent<IconVertex>>()
            .add_event::<RemoveComponentEvent<IconEdge>>()
            .add_event::<RemoveComponentEvent<IconFace>>()
            .add_event::<RemoveComponentEvent<IconFrame>>()
            .add_event::<RemoveComponentEvent<ShapeName>>()
            .add_event::<RemoveComponentEvent<AnimFrame>>()
            .add_event::<RemoveComponentEvent<AnimRotation>>()
            .add_event::<RemoveComponentEvent<PaletteColor>>()
            .add_event::<RemoveComponentEvent<BackgroundSkinColor>>()
            .add_event::<RemoveComponentEvent<FaceColor>>()
            .add_event::<RemoveComponentEvent<NetTransform>>()
            // Insert Component Systems
            .add_systems(Update, network::insert_file_component_events)
            .add_systems(Update, network::insert_changelist_entry_events)
            .add_systems(Update, network::insert_vertex_events)
            .add_systems(Update, network::insert_edge_events)
            .add_systems(Update, network::insert_face_events)
            .add_systems(Update, network::insert_icon_vertex_events)
            .add_systems(Update, network::insert_icon_edge_events)
            .add_systems(Update, network::insert_icon_face_events)
            .add_systems(Update, network::insert_icon_frame_events)
            .add_systems(Update, network::insert_owned_by_file_events)
            .add_systems(Update, network::insert_file_type_events)
            .add_systems(Update, network::insert_animation_events)
            .add_systems(Update, network::insert_palette_events)
            .add_systems(Update, network::insert_skin_events)
            .add_systems(Update, network::insert_model_events)
            .add_systems(Update, network::insert_shape_name_events)
            // Remove Component Systems
            // todo... possibly need to ensure one of these systems runs BEFORE `despawn_entity_events`
            .add_systems(Update, network::remove_file_component_events)
            .add_systems(Update, network::remove_shape_component_events)
            .add_systems(Update, network::remove_animation_component_events)
            .add_systems(Update, network::remove_color_component_events)
            .add_systems(Update, network::remove_model_component_events)
            .add_systems(Update, network::remove_icon_component_events)
            // Resources
            .init_resource::<ComponentWaitlist>()
            // UI Configuration
            .init_resource::<UiState>()
            .init_resource::<NamingBarState>()
            .init_resource::<FrameInspectBarState>()
            .insert_resource(file_manager)
            .init_resource::<FileActions>()
            .init_resource::<TabManager>()
            .add_systems(Update, ui::update)
            // Canvas Config
            .init_resource::<VertexManager>()
            .init_resource::<EdgeManager>()
            .init_resource::<FaceManager>()
            .init_resource::<AnimationManager>()
            .init_resource::<PaletteManager>()
            .init_resource::<CameraManager>()
            .init_resource::<InputManager>()
            .init_resource::<SkinManager>()
            .init_resource::<ModelManager>()
            .init_resource::<IconManager>()
            .init_resource::<Compass>()
            .init_resource::<Grid>()
            .init_resource::<Canvas>()
            .add_systems(Startup, canvas::setup)
            .add_systems(Update, canvas::update_camera)
            .add_systems(Update, canvas::update_tab_content_focus)
            .add_systems(Update, canvas::queue_resyncs)
            .add_systems(Update, TabManager::sync_tabs)
            .add_systems(Update, canvas::queue_shape_color_resync)
            .add_systems(Update, canvas::sync_compass)
            .add_systems(Update, canvas::sync_vertices)
            .add_systems(Update, canvas::sync_edges)
            .add_systems(Update, canvas::sync_faces)
            .add_systems(Update, canvas::update_animation)
            .add_systems(Update, canvas::update_icon)
            .add_systems(Update, canvas::update_palette)
            .add_systems(Update, canvas::process_faces)
            .add_systems(Update, canvas::update_selection_ui)
            .add_systems(Startup, canvas::input_startup)
            .add_systems(Update, canvas::input_update)
            .add_systems(Update, canvas::update_mouse_hover)
            .add_event::<ShapeColorResyncEvent>()
            // Draw
            .add_systems(Draw, draw)
            .add_systems(Draw, draw_vertices_and_edges);
    }
}
