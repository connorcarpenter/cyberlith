use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Startup, Update};

use game_engine::{
    http::CookieStore,
    kernel::KernelApp,
    render::{resources::WindowSettings, Draw},
    EnginePlugin,
};

use crate::{
    resources::{
        BackButtonClickedEvent, ForgotPasswordButtonClickedEvent, ForgotUsernameButtonClickedEvent,
        Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
        TextboxClickedEvent,
    },
    systems::{draw, resize, scene},
    ui,
};

pub struct LauncherApp {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl KernelApp for LauncherApp {
    fn init(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self
    where
        Self: Sized,
    {
        Self { cookie_store_opt }
    }
}

impl Plugin for LauncherApp {
    fn build(&self, app: &mut App) {

        #[cfg(feature = "autodriver")]
        app.add_plugins(crate::autodriver::AutodriverPlugin);

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
            .add_event::<BackButtonClickedEvent>()
            .add_event::<SubmitButtonClickedEvent>()
            .add_event::<ForgotUsernameButtonClickedEvent>()
            .add_event::<ForgotPasswordButtonClickedEvent>()
            .add_event::<TextboxClickedEvent>()
            // ui systems
            .add_systems(Startup, ui::setup)
            .add_systems(Update, ui::handle_events)
            .add_systems(Update, ui::process_requests)
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
