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
    systems::{cube_scene, draw, init_spinner, resize, session, walker_scene, world},
    resources::{match_lobbies::MatchLobbies, global_chat::GlobalChat, AssetCatalog, user_manager::UserManager},
    states::AppState,
    ui,
    ui::{
        events::{
            DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent,
            JoinMatchButtonClickedEvent, SettingsButtonClickedEvent, SubmitButtonClickedEvent, ResyncGlobalChatEvent, ResyncMatchLobbiesEvent,
        },
        UiCatalog,
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
            .init_resource::<GlobalChat>()
            .init_resource::<MatchLobbies>()
            .insert_resource(UiCatalog::new())
            .insert_resource(AssetCatalog::new())
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
            .add_systems(Draw, init_spinner::draw.run_if(in_state(AppState::Loading)))
            // Network Systems
            .add_systems(Update, world::world_spawn_entity_events)
            .add_systems(Update, world::world_main_insert_position_events)
            .add_systems(Update, world::world_main_insert_asset_ref_events)
            .add_systems(Update, world::world_alt1_insert_asset_ref_events)
            .add_systems(Update, session::session_load_asset_events)

            .add_systems(Update, session::recv_inserted_public_user_info_component)
            .add_systems(Update, session::recv_updated_public_user_info_component)
            .add_systems(Update, session::recv_removed_public_user_info_component)
            .add_systems(Update, session::recv_inserted_global_chat_component)
            .add_systems(Update, session::recv_inserted_match_lobby_component)
            .add_systems(Update, session::recv_removed_match_lobby_component)
            // Ui
            .add_systems(Update, ui::handle_events)
            .add_systems(Update, ui::handle_global_chat_events)
            .add_systems(Update, ui::handle_match_lobbies_events)
            .add_event::<ResyncGlobalChatEvent>()
            .add_event::<ResyncMatchLobbiesEvent>()
            .add_event::<HostMatchButtonClickedEvent>()
            .add_event::<JoinMatchButtonClickedEvent>()
            .add_event::<GlobalChatButtonClickedEvent>()
            .add_event::<DevlogButtonClickedEvent>()
            .add_event::<SettingsButtonClickedEvent>()
            .add_event::<SubmitButtonClickedEvent>();
    }
}
