use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, NetworkedEnginePlugin,
};

use super::systems::{network, scene};

pub fn run() {
    let mut app = App::default();

    app.add_plugins(NetworkedEnginePlugin)
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
        .add_systems(Update, network::world_spawn_entity_events)
        .add_systems(Update, network::world_main_insert_position_events)
        .add_systems(Update, network::world_main_insert_asset_ref_events)
        .add_systems(Update, network::world_alt1_insert_asset_ref_events);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}
