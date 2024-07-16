
use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::{EventReader, EventWriter},
    prelude::Query,
};

use game_engine::{
    asset::AssetManager,
    input::InputEvent,
    session::{
        components::{ChatMessage, User},
        SessionClient,
    },
    ui::{UiHandle, UiManager},
};

use crate::{
    resources::{lobby_manager::LobbyManager, chat_message_manager::ChatMessageManager},
    ui::{events::ResyncMessageListUiEvent, UiCatalog, UiKey},
};

pub(crate) fn handle_message_list_interaction_events(
    ui_catalog: Res<UiCatalog>,
    mut ui_manager: ResMut<UiManager>,
    mut session_client: SessionClient,
    mut message_manager: ResMut<ChatMessageManager>,
    lobby_manager: Res<LobbyManager>,
    mut input_events: EventReader<InputEvent>,
    mut resync_message_list_events: EventWriter<ResyncMessageListUiEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        if UiKey::GlobalChat == ui_catalog.get_ui_key(&current_ui_handle) {
            message_manager.handle_interaction_events(
                &mut ui_manager,
                &ui_catalog,
                &mut session_client,
                &lobby_manager,
                &mut input_events,
                &mut resync_message_list_events,
            );
        }
    };
}

pub(crate) fn handle_resync_message_list_ui_events(
    mut session_client: SessionClient,
    ui_catalog: Res<UiCatalog>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut message_manager: ResMut<ChatMessageManager>,
    lobby_manager: Res<LobbyManager>,
    user_q: Query<&User>,
    message_q: Query<&ChatMessage>,
    mut resync_message_list_events: EventReader<ResyncMessageListUiEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        if UiKey::GlobalChat == ui_catalog.get_ui_key(&current_ui_handle) {
            message_manager.handle_resync_events(
                &mut session_client,
                &mut ui_manager,
                &asset_manager,
                &lobby_manager,
                &user_q,
                &message_q,
                &mut resync_message_list_events,
            );
        }
    };
}

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}
