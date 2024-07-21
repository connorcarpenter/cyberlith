use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Update};

use game_engine::{
    http::CookieStore,
    kernel::KernelApp,
    NetworkedEnginePlugin,
    render::{Draw, resources::WindowSettings},
};

use crate::{
    inworld::InWorldPlugin,
    main_menu::MainMenuPlugin,
    states::AppState,
    systems::{
        draw, resize,
    },
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

        app.add_plugins(networked_engine_plugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            .add_plugins(InWorldPlugin)
            .add_plugins(MainMenuPlugin)

            // states
            .insert_state(AppState::Loading)

            // resize window listener
            .add_systems(Update, resize::handle_viewport_resize)

            // general drawing
            .add_systems(Draw, draw::draw);
    }
}
