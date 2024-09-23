use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Res, ResMut},
};
use bevy_state::state::{NextState, State};

use game_engine::{
    asset::{AssetLoadedEvent, AssetType},
    logging::info,
    ui::UiManager,
};

use game_app_common::AppState;

use crate::{
    resources::{
        asset_catalog::{on_asset_load, AssetCatalog},
        chat_message_manager::ChatMessageManager,
        lobby_manager::LobbyManager,
        user_manager::UserManager,
    },
    ui::{
        events::{
            GoToSubUiEvent, ResyncLobbyListUiEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent,
        },
        on_ui_load, UiCatalog,
    },
};

pub fn session_load_asset_events(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_manager: ResMut<UiManager>,
    mut ui_catalog: ResMut<UiCatalog>,
    mut user_manager: ResMut<UserManager>,
    mut asset_catalog: ResMut<AssetCatalog>,
    mut message_manager: ResMut<ChatMessageManager>,
    mut lobby_manager: ResMut<LobbyManager>,
    mut asset_loaded_event_reader: EventReader<AssetLoadedEvent>,
    mut resync_user_ui_events: EventWriter<ResyncUserListUiEvent>,
    mut resync_chat_message_ui_events: EventWriter<ResyncMessageListUiEvent>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyListUiEvent>,
    mut sub_ui_event_writer: EventWriter<GoToSubUiEvent>,
) {
    for event in asset_loaded_event_reader.read() {
        let asset_id = event.asset_id;
        let asset_type = event.asset_type;
        match asset_type {
            AssetType::Ui => {
                info!("received Asset Loaded Ui Event! (asset_id: {:?})", asset_id);
                let state = *state.get();
                on_ui_load(
                    state,
                    &mut next_state,
                    &mut ui_manager,
                    &mut ui_catalog,
                    &mut user_manager,
                    &mut message_manager,
                    &mut lobby_manager,
                    &mut sub_ui_event_writer,
                    &mut resync_user_ui_events,
                    &mut resync_chat_message_ui_events,
                    &mut resync_lobby_ui_events,
                    asset_id,
                );
            }
            _ => {
                on_asset_load(
                    &mut ui_manager,
                    &mut asset_catalog,
                    &mut resync_user_ui_events,
                    &mut resync_chat_message_ui_events,
                    &mut resync_lobby_ui_events,
                    asset_id,
                );
            }
        }
    }
}
