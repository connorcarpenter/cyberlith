use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use render_api::resources::WindowSettings;

use vortex_proto::{
    components::{
        ChangelistEntry, Edge3d, EntryKind, Face3d, FileSystemChild, FileSystemEntry,
        FileSystemRootChild, FileType, OwnedByFile, Vertex3d, VertexRoot,
    },
    protocol,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    config::ConfigPlugin,
    events::{InsertComponentEvent, LoginEvent},
    resources::{
        camera_manager::CameraManager, canvas::Canvas, global::Global,
        input_manager::InputManager, shape_manager::ShapeManager, shape_waitlist::ShapeWaitlist,
        tab_manager::TabManager, toolbar::Toolbar,
    },
    systems::{canvas, network, ui},
    ui::UiState,
};

pub struct VortexPlugin;

impl Plugin for VortexPlugin {
    fn build(&self, app: &mut App) {
        // setup Global
        let project_root_entity = app
            .world
            .spawn_empty()
            .insert(FileSystemParent::new())
            .insert(FileSystemUiState::new_root())
            .insert(FileSystemEntry::new("Project", EntryKind::Directory))
            .id();
        let global_resource = Global::new(project_root_entity);

        app
            // Add Config
            .add_plugins(ConfigPlugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Vortex".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Networking Plugin
            .add_plugins(ClientPlugin::new(ClientConfig::default(), protocol()))
            .add_event::<LoginEvent>()
            // Networking Systems
            .add_systems(Update, network::login)
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
                    network::remove_component_events.before(network::despawn_entity_events),
                    network::auth_granted_events,
                    network::auth_denied_events,
                    network::auth_reset_events,
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
            .add_event::<InsertComponentEvent<OwnedByFile>>()
            .add_event::<InsertComponentEvent<Edge3d>>()
            .add_event::<InsertComponentEvent<Face3d>>()
            .add_event::<InsertComponentEvent<FileType>>()
            .init_resource::<ShapeWaitlist>()
            .add_systems(Update, network::insert_fs_component_events)
            .add_systems(Update, network::insert_changelist_entry_events)
            .add_systems(Update, network::insert_vertex_events)
            .add_systems(Update, network::insert_edge_events)
            .add_systems(Update, network::insert_face_events)
            .add_systems(Update, network::insert_shape_events)
            // UI Configuration
            .init_resource::<UiState>()
            .insert_resource(global_resource)
            .init_resource::<TabManager>()
            .init_resource::<Toolbar>()
            .add_systems(Update, ui::update)
            // Canvas Config
            .init_resource::<ShapeManager>()
            .init_resource::<Canvas>()
            .init_resource::<CameraManager>()
            .init_resource::<InputManager>()
            .add_systems(Startup, canvas::setup)
            .add_systems(Update, canvas::update_camera)
            .add_systems(Update, canvas::sync_vertices)
            .add_systems(Update, canvas::process_faces)
            .add_systems(Update, canvas::update_select_line)
            .add_systems(Update, canvas::input)
            .add_systems(Update, canvas::update_mouse_hover);
    }
}
