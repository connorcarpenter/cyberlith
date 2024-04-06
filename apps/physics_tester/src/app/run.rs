use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    EnginePlugin,
};
use crate::app::global::Global;

use super::{draw, resize, startup::startup};

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Physics Tester".to_string(),
            min_size: (320, 180),
            max_size: None,
            ..Default::default()
        })
        // startup
        .add_systems(Startup, startup)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        // draw
        .add_systems(Draw, draw::draw);
    app.run();
}
