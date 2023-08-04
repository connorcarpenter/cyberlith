use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};
use render_api::resources::WindowSettings;
use vortex_proto::{
    components::{EntryKind, FileSystemEntry},
    protocol,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    config::ConfigPlugin,
    events::LoginEvent,
    resources::{
        camera_manager::CameraManager,
        canvas::Canvas,
        action_stack::ActionStack, canvas_manager::CanvasManager, global::Global,
        tab_manager::TabManager,
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
                    network::remove_component_events,
                    network::auth_granted_events,
                    network::auth_denied_events,
                    network::auth_reset_events,
                )
                    .in_set(ReceiveEvents),
            )
            // UI Configuration
            .insert_resource(UiState::new())
            .insert_resource(global_resource)
            .insert_resource(TabManager::new())
            .insert_resource(ActionStack::new())
            .add_system(ui::update)
            // Canvas Config
            .init_resource::<CanvasManager>()
            .init_resource::<Canvas>()
            .init_resource::<CameraManager>()
            .add_startup_system(canvas::setup)
            .add_system(canvas::step)
            .add_system(canvas::sync)
            .add_system(canvas::input);
    }
}
