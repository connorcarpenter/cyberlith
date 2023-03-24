use bevy_a11y::AccessibilityPlugin;
use bevy_app::App;
use bevy_asset::AssetPlugin;
use bevy_core::{FrameCountPlugin, TaskPoolPlugin, TypeRegistrationPlugin};
use bevy_core_pipeline::{clear_color::ClearColor, CorePipelinePlugin};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_input::InputPlugin;
use bevy_log::LogPlugin;
use bevy_render::{color::Color, texture::ImagePlugin, RenderPlugin};
use bevy_sprite::SpritePlugin;
use bevy_time::TimePlugin;
use bevy_transform::TransformPlugin;
use bevy_window::WindowPlugin;
use bevy_winit::WinitPlugin;

use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use cybl_nexus_proto::protocol;

mod resources;
mod systems;

use crate::app::systems::{events, init};

pub fn run() {
    App::default()
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
        .add_plugin(ClientPlugin::new(ClientConfig::default(), protocol()))
        // Background Color
        .insert_resource(ClearColor(Color::BLACK))
        // Startup System
        .add_startup_system(init)
        // Receive Client Events
        .add_systems(
            (
                events::connect_events,
                events::disconnect_events,
                events::reject_events,
                events::error_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
