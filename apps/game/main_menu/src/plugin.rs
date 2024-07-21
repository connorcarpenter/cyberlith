
use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{prelude::{in_state, not}, schedule::IntoSystemConfigs};

use game_engine::render::Draw;

use game_app_common::AppState;

use crate::{ui::UiPlugin, systems::session_component_events::SessionComponentEventsPlugin, resources::{user_manager::UserManager, selfhood_events::SelfhoodEvents, match_manager::MatchManager, lobby_manager::LobbyManager, chat_message_manager::ChatMessageManager, chat_message_events::ChatMessageEvents, asset_catalog::AssetCatalog}, systems};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UserManager>()
            .init_resource::<SelfhoodEvents>()
            .init_resource::<ChatMessageManager>()
            .init_resource::<ChatMessageEvents>()
            .init_resource::<LobbyManager>()
            .init_resource::<MatchManager>()
            .init_resource::<AssetCatalog>() // this seems to be Ui-specific
            // Ui
            .add_plugins(UiPlugin)
            // scene systems
            .add_systems(Startup, systems::cube_scene::setup)
            .add_systems(Update, systems::cube_scene::step.run_if(not(in_state(AppState::InGame))))
            .add_systems(Update, systems::resize::resync_on_resize)
            // Network Systems
            .add_systems(Update, systems::asset_events::session_load_asset_events)
            .add_plugins(SessionComponentEventsPlugin)
            // drawing loading spinner
            .add_systems(
                Draw,
                systems::initial_spinner::draw.run_if(in_state(AppState::Loading)),
            );
    }
}