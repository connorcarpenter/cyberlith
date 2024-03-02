use bevy_app::{App, Startup};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
};

use super::{systems::scene, resources};

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Cyberlith Launcher".to_string(),
            max_size: Some((1280, 720)),
            ..Default::default()
        })
        // Scene Systems
        .add_systems(Startup, scene::scene_setup)
        // .add_systems(Update, scene::scene_step)
        .add_systems(Draw, scene::scene_draw);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}
