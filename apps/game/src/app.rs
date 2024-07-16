use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{prelude::in_state, schedule::IntoSystemConfigs};

use game_engine::{
    http::CookieStore,
    kernel::KernelApp,
    render::{resources::WindowSettings, Draw},
    NetworkedEnginePlugin,
};

use crate::{
    ui::UiPlugin,
    resources::{
        selfhood_events::SelfhoodEvents,
        chat_message_manager::ChatMessageManager, lobby_manager::LobbyManager,
        user_manager::UserManager, AssetCatalog, chat_message_events::ChatMessageEvents,
    },
    states::AppState,
    systems::{
        asset_events, cube_scene, draw, initial_spinner, resize,
        session_component_events::SessionComponentEventsPlugin, walker_scene, world,
    },
};

pub struct GameApp {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl KernelApp for GameApp {
    fn init(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self
    where
        Self: Sized,
    {
        Self { cookie_store_opt }
    }
}

impl Plugin for GameApp {
    fn build(&self, app: &mut App) {
        let networked_engine_plugin = NetworkedEnginePlugin::new(self.cookie_store_opt.clone());

        app.add_plugins(networked_engine_plugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                min_size: (320, 180),
                max_size: None,
                ..Default::default()
            })
            .init_resource::<UserManager>()
            .init_resource::<SelfhoodEvents>()
            .init_resource::<ChatMessageManager>()
            .init_resource::<ChatMessageEvents>()
            .init_resource::<LobbyManager>()
            .init_resource::<AssetCatalog>()
            // states
            .insert_state(AppState::Loading)
            // scene systems
            .add_systems(Startup, cube_scene::setup)
            .add_systems(Update, cube_scene::step)
            .add_systems(Update, walker_scene::step)
            // resize window listener
            .add_systems(Update, resize::handle_viewport_resize)
            // general drawing
            .add_systems(Draw, draw::draw)
            // drawing loading spinner
            .add_systems(
                Draw,
                initial_spinner::draw.run_if(in_state(AppState::Loading)),
            )
            // Network Systems
            .add_systems(Update, world::world_spawn_entity_events)
            .add_systems(Update, world::world_main_insert_position_events)
            .add_systems(Update, world::world_main_insert_asset_ref_events)
            .add_systems(Update, world::world_alt1_insert_asset_ref_events)
            .add_systems(Update, asset_events::session_load_asset_events)
            .add_plugins(SessionComponentEventsPlugin)
            // Ui
            .add_plugins(UiPlugin);
    }
}
