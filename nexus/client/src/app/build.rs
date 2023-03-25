use bevy_a11y::AccessibilityPlugin;
use bevy_app::App;
use bevy_asset::AssetPlugin;
use bevy_core::{FrameCountPlugin, TaskPoolPlugin, TypeRegistrationPlugin};
use bevy_core_pipeline::CorePipelinePlugin;
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfigs};
use bevy_input::InputPlugin;
use bevy_log::LogPlugin;
use bevy_render::{texture::ImagePlugin, RenderPlugin};
use bevy_sprite::SpritePlugin;
use bevy_time::TimePlugin;
use bevy_transform::TransformPlugin;
use bevy_window::WindowPlugin;
use bevy_winit::WinitPlugin;

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use cybl_game_client::GameClientPlugin;
use cybl_nexus_proto::protocol;

use crate::app::systems::{context, network};

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        .add_plugin(TaskPoolPlugin::default())
        .add_plugin(TypeRegistrationPlugin::default())
        .add_plugin(FrameCountPlugin::default())
        .add_plugin(TimePlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(WindowPlugin::default())
        .add_plugin(AccessibilityPlugin)
        .add_plugin(AssetPlugin::default())
        .add_plugin(WinitPlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(ImagePlugin::default())
        .add_plugin(CorePipelinePlugin::default())
        .add_plugin(SpritePlugin::default())
        // Add Naia Client Plugin
        .add_plugin(NaiaClientPlugin::new(
            NaiaClientConfig::default(),
            protocol(),
        ))
        // Add Game Client Plugin
        .add_plugin(GameClientPlugin)
        // Add Context
        .add_plugin(context::ContextPlugin)
        .add_startup_systems(
            (
                cybl_game_client::setup,
                apply_system_buffers,
                context::setup,
            )
                .chain(),
        )
        // Startup System
        .add_startup_system(network::init)
        // Receive Client Events
        .add_systems(
            (
                network::connect_events,
                network::disconnect_events,
                network::reject_events,
                network::error_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        );
    app
}
