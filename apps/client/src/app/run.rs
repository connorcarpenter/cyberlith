use bevy_app::{App, Startup, Update};

use crate::app::resources::asset_store::AssetStore;
use crate::app::resources::global::Global;
use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
};

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
        .insert_resource(AssetStore::new("assets"))
        .add_systems(Startup, AssetStore::startup)
        .add_systems(Update, AssetStore::handle_metadata_tasks)
        .add_systems(Update, AssetStore::handle_load_asset_tasks)
        .add_systems(Update, AssetStore::handle_save_asset_tasks)
        .init_resource::<Global>()
        .add_systems(Update, network::handle_connection)
        .add_systems(Update, network::session_connect_events)
        .add_systems(Update, network::session_message_events)
        .add_systems(Update, network::session_request_events)
        .add_systems(Update, network::world_connect_events)
        .add_systems(Update, network::world_spawn_entity_events)
        .add_systems(Update, network::world_insert_component_events);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}
