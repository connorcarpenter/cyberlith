use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
};

use crate::app::resources::{global::Global, asset_ref_processor::AssetRefProcessor};

use super::systems::{network, scene};

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
        .init_resource::<Global>()
        .add_systems(Update, network::handle_connection)
        .add_systems(Update, network::session_connect_events)
        .add_systems(Update, network::session_message_events)
        .add_systems(Update, network::session_request_events)
        .add_systems(Update, network::world_connect_events)
        .add_systems(Update, network::world_spawn_entity_events)
        .add_systems(Update, network::world_insert_component_events)
        // todo: remove this
        .insert_resource(AssetRefProcessor::new())
        .add_systems(Update, network::handle_asset_loaded_events);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}
