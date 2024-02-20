
use bevy_app::{App, Startup, Update};

use game_engine::{render::{resources::WindowSettings, Draw}, EnginePlugin, wait_for_finish};

use super::{systems::{scene, network}, global::Global};

pub fn run() {
    let mut app = App::default();
    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Cyberlith".to_string(),
            max_size: Some((1280, 720)),
            ..Default::default()
        })
        // Scene Systems
        .add_systems(Startup, scene::scene_setup)
        .add_systems(Update, scene::scene_step)
        .add_systems(Draw, scene::scene_draw)
        // Network Systems
        .init_resource::<network::ApiTimer>()
        .add_systems(Update, network::handle_connection)
        .add_systems(Update, network::session_connect_events)
        .add_systems(Update, network::session_message_events)
        .add_systems(Update, network::session_request_events)
        .add_systems(Update, network::world_connect_events)
        .init_resource::<Global>();
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}