use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Startup, Update};

use game_engine::{
    kernel::KernelApp,
    render::{resources::WindowSettings, Draw},
    EnginePlugin,
};
use game_engine::http::CookieStore;

use crate::{
    ui::{ui_handle_events, ui_setup},
    resources::{
        TextboxClickedEvent, Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
    },
    systems::{backend, draw, resize, scene},
};

pub struct LauncherApp {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl KernelApp for LauncherApp {
    fn init(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self
    where
        Self: Sized,
    {
        Self {
            cookie_store_opt,
        }
    }
}

impl Plugin for LauncherApp {
    fn build(&self, app: &mut App) {

        let engine_plugin = EnginePlugin::new(self.cookie_store_opt.clone());

        app.add_plugins(engine_plugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith Launcher".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            // global resource
            .init_resource::<Global>()
            // events
            .add_event::<LoginButtonClickedEvent>()
            .add_event::<RegisterButtonClickedEvent>()
            .add_event::<SubmitButtonClickedEvent>()
            .add_event::<TextboxClickedEvent>()
            // ui systems
            .add_systems(Startup, ui_setup)
            .add_systems(Update, ui_handle_events)
            .add_systems(Update, backend::backend_step)
            // scene systems
            .add_systems(Startup, scene::scene_setup)
            .add_systems(Update, scene::scene_step)
            //.add_systems(Update, gamepad_testing::gamepad_testing_system)
            // viewport resize
            .add_systems(Update, resize::handle_viewport_resize)
            // draw
            .add_systems(Draw, draw::draw);
    }
}
