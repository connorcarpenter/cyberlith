use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    prelude::not,
    schedule::IntoSystemConfigs,
};
use bevy_state::condition::in_state;

use game_engine::render::Draw;

use game_app_common::AppState;

use crate::{
    resources::{
        asset_catalog::AssetCatalog, chat_message_events::ChatMessageEvents,
        chat_message_manager::ChatMessageManager, lobby_manager::LobbyManager,
        match_manager::MatchManager, selfhood_events::SelfhoodEvents, user_manager::UserManager,
    },
    systems,
    systems::session_component_events::SessionComponentEventsPlugin,
    ui::UiPlugin,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserManager>()
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
            .add_systems(
                Update,
                systems::cube_scene::step.run_if(not(in_state(AppState::InGame))),
            )
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
