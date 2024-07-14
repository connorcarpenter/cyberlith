use bevy_ecs::{
    event::{EventReader, EventWriter},
    schedule::{NextState, State},
    system::{Query, Res, ResMut},
};

use game_engine::{
    asset::{AssetLoadedEvent, AssetManager, AssetType},
    logging::info,
    session::{
        components::{LobbyPublic, MessagePublic, UserPublic},
        SessionInsertComponentEvent, SessionRemoveComponentEvent, SessionUpdateComponentEvent,
    },
    ui::UiManager,
};

use crate::{
    resources::{
        global_chat::GlobalChat, match_lobbies::MatchLobbies, on_asset_load,
        user_manager::UserManager, AssetCatalog,
    },
    states::AppState,
    ui::{
        events::{ResyncGlobalChatEvent, ResyncMatchLobbiesEvent, ResyncPublicUserInfoEvent},
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
    mut global_chat_messages: ResMut<GlobalChat>,
    mut match_lobbies: ResMut<MatchLobbies>,
    mut event_reader: EventReader<AssetLoadedEvent>,
    mut resync_user_public_info_events: EventWriter<ResyncPublicUserInfoEvent>,
    mut resync_global_chat_events: EventWriter<ResyncGlobalChatEvent>,
    mut resync_match_lobbies_events: EventWriter<ResyncMatchLobbiesEvent>,
) {
    for event in event_reader.read() {
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
                    &mut global_chat_messages,
                    &mut match_lobbies,
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
