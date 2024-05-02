use std::sync::{Arc, RwLock};
use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::system::ResMut;

use asset_loader::AssetPlugin;
use filesystem::FileSystemPlugin;
use input::{Input, InputPlugin};
use kernel::http::CookieStore;
use kernel::KernelPlugin;
use render_api::RenderApiPlugin;
use ui_render::UiRenderPlugin;
use ui_runner::UiPlugin;

use crate::{
    asset_cache::{AssetCache, AssetLoadedEvent},
    embedded_asset::handle_embedded_asset_event,
    renderer::RendererPlugin,
};

pub struct EnginePlugin {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl EnginePlugin {
    pub fn new(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self {
        Self { cookie_store_opt }
    }
}

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {

        let kernel_plugin = KernelPlugin::new(self.cookie_store_opt.clone());

        app
            // Add Render Plugins
            .add_plugins(RenderApiPlugin)
            .add_plugins(RendererPlugin)
            // Add misc crates Plugins
            .add_plugins(kernel_plugin)
            .add_plugins(InputPlugin)
            .add_plugins(AssetPlugin)
            .add_plugins(UiPlugin)
            .add_plugins(UiRenderPlugin)
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
