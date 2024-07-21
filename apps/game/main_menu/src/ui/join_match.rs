use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::{EventReader, EventWriter},
    prelude::Query,
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
    logging::info,
    session::{
        channels, components::{Lobby, User},
        messages,
        SessionClient,
    },
    ui::{UiHandle, UiManager},
};

use crate::{ui::{events::{GoToSubUiEvent, LobbyListItemClickedEvent, ResyncLobbyListUiEvent}, go_to_sub_ui, UiCatalog, UiKey}, resources::lobby_manager::LobbyManager};

pub(crate) fn handle_join_match_input_events(
    ui_catalog: Res<UiCatalog>,
    ui_manager: Res<UiManager>,
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_lobby_list_ui_events: EventWriter<ResyncLobbyListUiEvent>,
    mut input_events: EventReader<InputEvent>,
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
        let ui_key = ui_catalog.get_ui_key(&current_ui_handle);
        if ui_key == UiKey::JoinMatch {
            handle_join_match_input_events_impl(
                &mut lobby_manager,
                &mut resync_lobby_list_ui_events,
                &mut input_events,
            );
        }
    };
}

pub(crate) fn handle_join_match_click_events(
    ui_catalog: Res<UiCatalog>,
    ui_manager: Res<UiManager>,
    mut session_client: SessionClient,
    mut resync_lobby_list_ui_events: EventWriter<ResyncLobbyListUiEvent>,
    mut sub_ui_event_writer: EventWriter<GoToSubUiEvent>,
    mut click_events: EventReader<LobbyListItemClickedEvent>,
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
        let ui_key = ui_catalog.get_ui_key(&current_ui_handle);
        if ui_key == UiKey::JoinMatch {
            handle_join_match_click_events_impl(
                &mut session_client,
                &mut resync_lobby_list_ui_events,
                &mut sub_ui_event_writer,
                &mut click_events,
            );
        }
    };
}

pub(crate) fn handle_resync_lobby_list_ui_events(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut session_client: SessionClient,
    mut lobby_manager: ResMut<LobbyManager>,
    user_q: Query<&User>,
    lobby_q: Query<&Lobby>,
    mut resync_lobby_list_ui_events: EventReader<ResyncLobbyListUiEvent>,
) {
    handle_resync_lobby_list_ui_events_impl(
        &mut lobby_manager,
        &mut ui_manager,
        &asset_manager,
        &mut session_client,
        &user_q,
        &lobby_q,
        &mut resync_lobby_list_ui_events,
    );
}

fn handle_join_match_input_events_impl(
    lobby_manager: &mut LobbyManager,
    resync_lobby_ui_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    input_events: &mut EventReader<InputEvent>,
) {
    let mut should_resync = false;

    for event in input_events.read() {
        match event {
            // TODO this probably doesn't belong here! this is where it is required to be selecting the textbox!!!
            InputEvent::KeyPressed(Key::I, _) => {
                info!("Scrolling Up");
                lobby_manager.scroll_up();
                should_resync = true;
            }
            InputEvent::KeyPressed(Key::J, _) => {
                info!("Scrolling Down");
                lobby_manager.scroll_down();
                should_resync = true;
            }
            InputEvent::KeyPressed(Key::Enter, _modifiers) => {
                // TODO: enter into lobby?
            }
            _ => {}
        }
    }

    if should_resync {
        resync_lobby_ui_events.send(ResyncLobbyListUiEvent);
    }
}

fn handle_join_match_click_events_impl(
    session_client: &mut SessionClient,
    resync_lobby_ui_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    sub_ui_event_writer: &mut EventWriter<GoToSubUiEvent>,
    click_events: &mut EventReader<LobbyListItemClickedEvent>,
) {
    let mut should_resync = false;

    for event in click_events.read() {

        if !should_resync { // prevent multiple clicks

            let lobby_id = event.lobby_id();
            info!("Joining lobby: {:?}", lobby_id.to_u16());

            let message = messages::MatchLobbyJoin::new(lobby_id);
            session_client.send_message::<channels::ClientActionsChannel, _>(&message);

            go_to_sub_ui(sub_ui_event_writer, UiKey::MessageList);

            should_resync = true;
        }
    }

    if should_resync {
        resync_lobby_ui_events.send(ResyncLobbyListUiEvent);
    }
}

fn handle_resync_lobby_list_ui_events_impl(
    lobby_manager: &mut LobbyManager,
    ui_manager: &mut UiManager,
    asset_manager: &AssetManager,
    session_client: &mut SessionClient,
    user_q: &Query<&User>,
    lobby_q: &Query<&Lobby>,
    resync_lobby_ui_events: &mut EventReader<ResyncLobbyListUiEvent>,
) {
    let mut should_resync = false;
    for _resync_event in resync_lobby_ui_events.read() {
        should_resync = true;
    }

    if should_resync {
        lobby_manager.sync_with_collection(
            session_client,
            ui_manager,
            asset_manager,
            user_q,
            lobby_q,
        );
    }
}

pub fn on_enter_state(resync_lobby_list_ui_event_writer: &mut EventWriter<ResyncLobbyListUiEvent>) {
    resync_lobby_list_ui_event_writer.send(ResyncLobbyListUiEvent);
}

pub fn on_leave_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}
