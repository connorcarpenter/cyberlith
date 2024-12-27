use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Startup};
use bevy_ecs::system::ResMut;
use bevy_state::app::StatesPlugin;

use asset_cache::AssetCachePlugin;
use asset_loader::AssetPlugin;
use filesystem::FileSystemPlugin;
use input::{Input, InputPlugin};
use kernel::{http::CookieStore, KernelPlugin};
use render_api::RenderApiPlugin;
use ui_render::UiRenderPlugin;
use ui_runner::UiPlugin;

use crate::renderer::RendererPlugin;

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

        app.add_plugins(kernel_plugin)
            // Add Render Plugins
            .add_plugins(RenderApiPlugin)
            .add_plugins(RendererPlugin)
            // Add misc crates Plugins
            .add_plugins(InputPlugin)
            .add_plugins(AssetPlugin)
            .add_plugins(UiPlugin)
            .add_plugins(UiRenderPlugin)
            .add_plugins(FileSystemPlugin)
            .add_plugins(StatesPlugin)
            .add_plugins(AssetCachePlugin)
            // startup system
            .add_systems(Startup, engine_startup);
    }
}

fn engine_startup(mut input: ResMut<Input>) {
    input.set_enabled(true);
}
