use bevy_app::{App, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemConfigs};

use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use render_api::resources::WindowSettings;

use vortex_proto::{
    components::{
        ChangelistEntry, Edge3d, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild,
        OwnedByTab, Vertex3d, VertexRoot,
    },
    protocol,
};
use vortex_proto::components::FileType;

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    config::ConfigPlugin,
    events::{InsertComponentEvent, LoginEvent},
    resources::{
        action_stack::ActionStack, camera_manager::CameraManager, canvas::Canvas, global::Global,
        input_manager::InputManager, shape_waitlist::ShapeWaitlist, tab_manager::TabManager,
        shape_manager::ShapeManager,
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
            .add_plugin(ConfigPlugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Vortex".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Networking Plugin
            .add_plugin(ClientPlugin::new(ClientConfig::default(), protocol()))
            .add_event::<LoginEvent>()
            // Networking Systems
            .add_system(network::login)
            .add_systems(
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
            .add_event::<InsertComponentEvent<OwnedByTab>>()
            .add_event::<InsertComponentEvent<Edge3d>>()
            .add_event::<InsertComponentEvent<FileType>>()
            .init_resource::<ShapeWaitlist>()
            .add_system(network::insert_fs_component_events)
            .add_system(network::insert_changelist_entry_events)
            .add_system(network::insert_vertex_events)
            // UI Configuration
            .init_resource::<UiState>()
            .insert_resource(global_resource)
            .init_resource::<TabManager>()
            .init_resource::<ActionStack>()
            .add_system(ui::update)
            // Canvas Config
            .init_resource::<ShapeManager>()
            .init_resource::<Canvas>()
            .init_resource::<CameraManager>()
            .init_resource::<InputManager>()
            .add_startup_system(canvas::setup)
            .add_system(canvas::update_camera)
            .add_system(canvas::sync_vertices)
            .add_system(canvas::update_select_line)
            .add_system(canvas::input)
            .add_system(canvas::update_mouse_hover);
    }
}
