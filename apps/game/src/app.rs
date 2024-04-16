use bevy_app::{App, Plugin, Startup, Update};

use game_engine::{
    kernel::KernelApp,
    render::{resources::WindowSettings, Draw},
    NetworkedEnginePlugin,
};

use super::systems::{keyboard_input, network, scene};

pub struct GameApp;

impl KernelApp for GameApp {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self
    }
}

impl Plugin for GameApp {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetworkedEnginePlugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            // other input
            .add_systems(Update, keyboard_input::process)
            // Scene Systems
            .add_systems(Startup, scene::scene_setup)
            .add_systems(Update, scene::scene_step)
            .add_systems(Update, scene::handle_viewport_resize)
            .add_systems(Draw, scene::scene_draw)
            // Network Systems
            .add_systems(Update, network::world_spawn_entity_events)
            .add_systems(Update, network::world_main_insert_position_events)
            .add_systems(Update, network::world_main_insert_asset_ref_events)
            .add_systems(Update, network::world_alt1_insert_asset_ref_events);
    }
}
