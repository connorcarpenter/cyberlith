use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::system::ResMut;
use bevy_log::LogPlugin;

use asset_loader::AssetPlugin;
use bevy_http_client::HttpClientPlugin;
use filesystem::FileSystemPlugin;
use input::{Input, InputPlugin};
use render_api::RenderApiPlugin;

use crate::{
    asset_cache::{AssetCache, AssetLoadedEvent},
    embedded_asset::handle_embedded_asset_event,
    renderer::RendererPlugin,
};

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy Plugins
            .add_plugins(LogPlugin::default())
            // Add Render Plugins
            .add_plugins(RenderApiPlugin)
            .add_plugins(RendererPlugin)
            // Add misc crates Plugins
            .add_plugins(InputPlugin)
            .add_plugins(AssetPlugin)
            .add_plugins(HttpClientPlugin)
            .add_plugins(FileSystemPlugin)
            .add_systems(Startup, engine_startup)
            // asset cache stuff, todo: maybe refactor out?
            .insert_resource(AssetCache::new("assets"))
            .add_event::<AssetLoadedEvent>()
            .add_systems(Update, AssetCache::handle_save_asset_tasks)
            // embedded asset
            .add_systems(Update, handle_embedded_asset_event);
    }
}

fn engine_startup(mut input: ResMut<Input>) {
    input.set_enabled(true);
}
