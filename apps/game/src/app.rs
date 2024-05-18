use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Startup, Update};

use game_engine::{
    kernel::KernelApp,
    render::{resources::WindowSettings, Draw},
    NetworkedEnginePlugin,
    http::CookieStore,
};

use super::systems::{keyboard_input, network, walker_scene, init_spinner, cube_scene, resize, draw};

pub struct GameApp {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl KernelApp for GameApp {
    fn init(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self
    where
        Self: Sized,
    {
        Self {
            cookie_store_opt,
        }
    }
}

impl Plugin for GameApp {
    fn build(&self, app: &mut App) {
        let networked_engine_plugin = NetworkedEnginePlugin::new(self.cookie_store_opt.clone());

        app.add_plugins(networked_engine_plugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            // other input
            .add_systems(Update, keyboard_input::process)
            // Scene Systems
            //.add_systems(Startup, walker_scene::scene_setup)
            .add_systems(Draw, init_spinner::draw)
            .add_systems(Update, walker_scene::step)
            .add_systems(Startup, cube_scene::setup)
            .add_systems(Update, cube_scene::step)
            .add_systems(Update, resize::handle_viewport_resize)
            .add_systems(Draw, draw::draw)
            // Network Systems
            .add_systems(Update, network::world_spawn_entity_events)
            .add_systems(Update, network::world_main_insert_position_events)
            .add_systems(Update, network::world_main_insert_asset_ref_events)
            .add_systems(Update, network::world_alt1_insert_asset_ref_events);
    }
}
