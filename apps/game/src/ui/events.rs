use std::time::Duration;

use bevy_ecs::{prelude::Query, event::{Event, EventReader}, change_detection::{Res, ResMut}};

use game_engine::{ui::UiManager, session::{SessionClient, components::{ChatMessage, Lobby, User}}, input::{GamepadRumbleIntensity, Input, InputEvent, RumbleManager}, asset::AssetManager};

use crate::{ui::{main_menu, UiCatalog, UiKey}, resources::{user_manager::UserManager, lobby_manager::LobbyManager, chat_message_manager::ChatMessageManager}};

#[derive(Event, Default)]
pub struct HostMatchButtonClickedEvent;

#[derive(Event, Default)]
pub struct JoinMatchButtonClickedEvent;

#[derive(Event, Default)]
pub struct GlobalChatButtonClickedEvent;

#[derive(Event, Default)]
pub struct DevlogButtonClickedEvent;

#[derive(Event, Default)]
pub struct SettingsButtonClickedEvent;

#[derive(Event, Default)]
pub struct SubmitButtonClickedEvent;

// UI events

#[derive(Event, Default)]
pub struct ResyncMainMenuUiEvent;

#[derive(Event, Default)]
pub struct ResyncUserListUiEvent;

#[derive(Event, Default)]
pub struct ResyncMessageListUiEvent {
    maintain_scroll: bool,
}

impl ResyncMessageListUiEvent {
    pub fn new(maintain_scroll: bool) -> Self {
        Self { maintain_scroll }
    }
    pub fn maintain_scroll(&self) -> bool {
        self.maintain_scroll
    }
}

#[derive(Event, Default)]
pub struct ResyncLobbyListUiEvent;

// event handling systems

pub(crate) fn handle_ui_interaction_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    mut rumble_manager: ResMut<RumbleManager>,

    mut host_match_btn_rdr: EventReader<HostMatchButtonClickedEvent>,
    mut join_match_btn_rdr: EventReader<JoinMatchButtonClickedEvent>,
    mut global_chat_btn_rdr: EventReader<GlobalChatButtonClickedEvent>,
    mut devlog_btn_rdr: EventReader<DevlogButtonClickedEvent>,
    mut settings_btn_rdr: EventReader<SettingsButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    main_menu::handle_ui_interaction_events(
        &mut ui_manager,
        &ui_catalog,
        &mut host_match_btn_rdr,
        &mut join_match_btn_rdr,
        &mut global_chat_btn_rdr,
        &mut devlog_btn_rdr,
        &mut settings_btn_rdr,
        &mut should_rumble,
    );

    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        match ui_catalog.get_ui_key(&current_ui_handle) {
            UiKey::MainMenu => panic!("invalid sub-ui"),
            UiKey::HostMatch | UiKey::JoinMatch | UiKey::GlobalChat => {
                // handling these in another method
            }
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    };

    // handle rumble
    if should_rumble {
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::strong_motor(0.4),
            );
        }
    }

    // drain all events
    for _ in host_match_btn_rdr.read() {}
    for _ in join_match_btn_rdr.read() {}
    for _ in global_chat_btn_rdr.read() {}
    for _ in devlog_btn_rdr.read() {}
    for _ in settings_btn_rdr.read() {}
}

pub(crate) fn handle_resync_main_menu_ui_events(
    mut resync_main_menu_ui_events: EventReader<ResyncMainMenuUiEvent>,
) {
    let mut resync = false;
    for _ in resync_main_menu_ui_events.read() {
        resync = true;
    }
    if resync {
        todo!();
    }
}

pub(crate) fn handle_resync_user_list_ui_events(
    mut user_manager: ResMut<UserManager>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    user_q: Query<&User>,
    mut resync_user_public_info_events: EventReader<ResyncUserListUiEvent>,
) {
    let mut resync = false;
    for _ in resync_user_public_info_events.read() {
        resync = true;
    }
    if resync {
        user_manager.sync_with_collection(&mut ui_manager, &asset_manager, &user_q);
    }
}

pub(crate) fn handle_resync_message_list_ui_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,
    mut message_manager: ResMut<ChatMessageManager>,
    user_q: Query<&User>,
    message_q: Query<&ChatMessage>,
    mut input_events: EventReader<InputEvent>,
    mut resync_global_chat_events: EventReader<ResyncMessageListUiEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        if UiKey::GlobalChat == ui_catalog.get_ui_key(&current_ui_handle) {
            message_manager.handle_events(
                &mut ui_manager,
                &ui_catalog,
                &asset_manager,
                &mut session_client,
                &mut input_events,
                &mut resync_global_chat_events,
                &user_q,
                &message_q,
                &mut should_rumble,
            );
        }
    };

    // handle rumble
    if should_rumble {
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::strong_motor(0.4),
            );
        }
    }
}

pub(crate) fn handle_resync_lobby_list_ui_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,
    mut lobby_manager: ResMut<LobbyManager>,
    user_q: Query<&User>,
    lobby_q: Query<&Lobby>,
    mut submit_btn_rdr: EventReader<SubmitButtonClickedEvent>,
    mut input_events: EventReader<InputEvent>,
    mut resync_match_lobbies_events: EventReader<ResyncLobbyListUiEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        let ui_key = ui_catalog.get_ui_key(&current_ui_handle);
        match ui_key {
            UiKey::HostMatch => {
                lobby_manager.handle_host_match_events(
                    &mut ui_manager,
                    &ui_catalog,
                    &mut session_client,
                    &mut submit_btn_rdr,
                    &mut should_rumble,
                );
            }
            UiKey::JoinMatch => {
                lobby_manager.handle_join_match_events(
                    &mut ui_manager,
                    &asset_manager,
                    &mut session_client,
                    &user_q,
                    &lobby_q,
                    &mut input_events,
                    &mut resync_match_lobbies_events,
                    &mut should_rumble,
                );
            }
            _ => {
                // handled elsewhere
            }
        }
    };

    // handle rumble
    if should_rumble {
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::strong_motor(0.4),
            );
        }
    }
}