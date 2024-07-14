use bevy_ecs::{
    event::{EventReader, EventWriter},
    schedule::{NextState, State},
    system::{Res, ResMut},
};

use game_engine::{
    asset::{AssetLoadedEvent, AssetType},
    logging::info,
    ui::UiManager,
};

use crate::{
    resources::{
        message_manager::MessageManager, lobby_manager::LobbyManager, on_asset_load,
        user_manager::UserManager, AssetCatalog,
    },
    states::AppState,
    ui::{
        events::{ResyncLobbyGlobalEvent, ResyncMatchLobbiesEvent, ResyncPublicUserInfoEvent},
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
    mut message_manager: ResMut<MessageManager>,
    mut lobby_manager: ResMut<LobbyManager>,
    mut asset_loaded_event_reader: EventReader<AssetLoadedEvent>,
    mut resync_user_public_info_events: EventWriter<ResyncPublicUserInfoEvent>,
    mut resync_global_chat_events: EventWriter<ResyncLobbyGlobalEvent>,
    mut resync_match_lobbies_events: EventWriter<ResyncMatchLobbiesEvent>,
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
                    &mut resync_user_public_info_events,
                    &mut resync_global_chat_events,
                    &mut resync_match_lobbies_events,
                    asset_id,
                );
            }
            _ => {
                on_asset_load(
                    &mut ui_manager,
                    &mut asset_catalog,
                    &mut resync_user_public_info_events,
                    &mut resync_global_chat_events,
                    &mut resync_match_lobbies_events,
                    asset_id,
                );
            }
        }
    }
}
