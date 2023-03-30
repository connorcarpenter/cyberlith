use bevy_app::App;
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfigs};
use bevy_log::LogPlugin;

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use game_client::GameClientPlugin;
use os_proto::protocol;
use render_api::{RenderApiPlugin, Window};

use crate::app::systems::{context, network};

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Add Context Plugin
        .add_plugin(context::ContextPlugin)
        .add_startup_systems((game_client::setup, apply_system_buffers, context::setup).chain())
        // Render API Plugin
        .add_plugin(RenderApiPlugin)
        // TODO: find out how to get window height & width
        .insert_resource(Window::new(1280, 720))
        // Add Renderer Plugin
        .add_plugin(context::RendererPlugin)
        // Add Naia Client Plugin
        .add_plugin(NaiaClientPlugin::new(
            NaiaClientConfig::default(),
            protocol(),
        ))
        // Add Game Client Plugin
        .add_plugin(GameClientPlugin)
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
