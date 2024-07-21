use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Update};

use game_engine::{
    http::CookieStore,
    kernel::KernelApp,
    NetworkedEnginePlugin,
    render::{Draw, resources::WindowSettings},
};

use game_app_common::CommonPlugin;
use game_app_inworld::InWorldPlugin;

#[cfg(feature = "no_odst")]
use game_app_main_menu::MainMenuPlugin;

use crate::systems::{
        draw, resize,
    };

pub struct GameApp {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl KernelApp for GameApp {
    fn init(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self
    where
        Self: Sized,
    {
        Self { cookie_store_opt }
    }
}

impl Plugin for GameApp {
    fn build(&self, app: &mut App) {
        let networked_engine_plugin = NetworkedEnginePlugin::new(self.cookie_store_opt.clone());

        #[cfg(feature = "no_odst")]
        app.add_plugins(MainMenuPlugin);

        app.add_plugins(networked_engine_plugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            .add_plugins(CommonPlugin)
            .add_plugins(InWorldPlugin)

            // handle resizes
            .add_systems(Update, resize::handle_viewport_resize)

            // general drawing
            .add_systems(Draw, draw::draw);
    }
}
