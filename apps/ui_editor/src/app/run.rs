use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    EnginePlugin,
};

use super::setup;

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "UI Editor".to_string(),
            min_size: (320, 180),
            max_size: None,
            ..Default::default()
        })
        // Scene Systems
        .add_systems(Startup, setup::setup_scene)
        .add_systems(Startup, setup::setup_ui)
        .add_systems(Draw, setup::scene_draw)
        .add_systems(Update, setup::handle_viewport_resize);
    app.run();
}
