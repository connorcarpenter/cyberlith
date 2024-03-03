use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
};

use super::systems::scene;

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Cyberlith Launcher".to_string(),
            min_size: (320, 180),
            max_size: None,
            ..Default::default()
        })
        // Scene Systems
        .add_systems(Startup, scene::scene_setup)
        .add_systems(Update, scene::handle_viewport_resize)
        .add_systems(Draw, scene::scene_draw);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}
