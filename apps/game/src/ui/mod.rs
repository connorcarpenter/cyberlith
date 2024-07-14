pub mod events;

mod ui_catalog;
pub use ui_catalog::UiCatalog;

mod main_menu;

use std::time::Duration;

use bevy_ecs::{
    event::{EventReader, EventWriter},
    prelude::NextState,
    system::{Res, Query, ResMut},
};

use game_engine::{
    asset::{AssetId, AssetManager},
    input::{GamepadRumbleIntensity, Input, InputEvent, RumbleManager},
    session::{SessionClient, components::{LobbyPublic, MessagePublic, UserPublic}},
    ui::{UiHandle, UiManager},
};

use crate::{
    resources::{match_lobbies::MatchLobbies, global_chat::GlobalChat, user_manager::UserManager},
    states::AppState,
    ui::events::{
        DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent, ResyncMatchLobbiesEvent,
        JoinMatchButtonClickedEvent, SettingsButtonClickedEvent, SubmitButtonClickedEvent, ResyncGlobalChatEvent, ResyncPublicUserInfoEvent,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UiKey {
    MainMenu,
    UserListItem,

    HostMatch,

    JoinMatch,
    JoinMatchLobbyItem,

    GlobalChat,
    GlobalChatDayDivider,
    GlobalChatUsernameAndMessage,
    GlobalChatMessage,

    Devlog,

    Settings,
}

pub(crate) fn on_ui_load(
    state: AppState,
    next_state: &mut NextState<AppState>,
    ui_manager: &mut UiManager,
    ui_catalog: &mut UiCatalog,
    user_manager: &mut UserManager,
    global_chat_messages: &mut GlobalChat,
    match_lobbies: &mut MatchLobbies,
    resync_user_public_info_events: &mut EventWriter<ResyncPublicUserInfoEvent>,
    resync_global_chat_events: &mut EventWriter<ResyncGlobalChatEvent>,
    resync_match_lobbies_events: &mut EventWriter<ResyncMatchLobbiesEvent>,
    asset_id: AssetId,
) {
    let ui_handle = UiHandle::new(asset_id);
    if !ui_catalog.has_ui_key(&ui_handle) {
        panic!("ui is not registered in catalog");
    }
    let ui_key = ui_catalog.get_ui_key(&ui_handle);

    match ui_key {
        UiKey::MainMenu => main_menu::on_load(
            state,
            next_state,
            ui_catalog,
            ui_manager,
            user_manager
        ),
        UiKey::UserListItem => user_manager.on_load_user_list_item_ui(
            ui_catalog,
            resync_user_public_info_events,
        ),

        UiKey::HostMatch => match_lobbies.on_load_host_match_ui(
            ui_catalog,
            ui_manager
        ),

        UiKey::JoinMatch => match_lobbies.on_load_lobby_list_ui(
            ui_catalog,
            ui_manager,
            resync_match_lobbies_events,
        ),
        UiKey::JoinMatchLobbyItem => match_lobbies.on_load_lobby_item_ui(
            ui_catalog,
            resync_match_lobbies_events,
        ),

        UiKey::GlobalChat => global_chat_messages.on_load_container_ui(
            ui_catalog,
            ui_manager,
            resync_global_chat_events,
        ),
        UiKey::GlobalChatDayDivider => global_chat_messages.on_load_day_divider_item_ui(
            ui_catalog,
            resync_global_chat_events,
        ),
        UiKey::GlobalChatUsernameAndMessage => global_chat_messages.on_load_username_and_message_item_ui(
            ui_catalog,
            resync_global_chat_events,
        ),
        UiKey::GlobalChatMessage => global_chat_messages.on_load_message_item_ui(
            ui_catalog,
            resync_global_chat_events,
        ),

        _ => {
            unimplemented!("ui not implemented");
        }
    }
}

pub(crate) fn handle_events(
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

    main_menu::handle_events(
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
            },
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

pub(crate) fn handle_user_public_info_events(
    mut user_manager: ResMut<UserManager>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    user_q: Query<&UserPublic>,
    mut resync_user_public_info_events: EventReader<ResyncPublicUserInfoEvent>,
) {
    let mut resync = false;
    for _ in resync_user_public_info_events.read() {
        resync = true;
    }
    if resync {
        user_manager.sync_with_collection(&mut ui_manager, &asset_manager, &user_q);
    }
}

pub(crate) fn handle_global_chat_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,
    mut global_chat: ResMut<GlobalChat>,
    user_q: Query<&UserPublic>,
    message_q: Query<&MessagePublic>,
    mut input_events: EventReader<InputEvent>,
    mut resync_global_chat_events: EventReader<ResyncGlobalChatEvent>,
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
            global_chat.handle_events(
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

pub(crate) fn handle_match_lobbies_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,
    mut match_lobbies: ResMut<MatchLobbies>,
    user_q: Query<&UserPublic>,
    lobby_q: Query<&LobbyPublic>,
    mut submit_btn_rdr: EventReader<SubmitButtonClickedEvent>,
    mut input_events: EventReader<InputEvent>,
    mut resync_match_lobbies_events: EventReader<ResyncMatchLobbiesEvent>,
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
                match_lobbies.handle_host_match_events(
                    &mut ui_manager,
                    &ui_catalog,
                    &mut session_client,
                    &mut submit_btn_rdr,
                    &mut should_rumble
                );
            }
            UiKey::JoinMatch => {
                match_lobbies.handle_join_match_events(
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

pub(crate) fn go_to_sub_ui(ui_manager: &mut UiManager, ui_catalog: &UiCatalog, sub_ui_key: UiKey) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("invalid sub-ui");
    }
    let sub_ui_handle = ui_catalog.get_ui_handle(sub_ui_key);
    if !ui_catalog.get_is_loaded(sub_ui_key) {
        panic!("ui not loaded");
    }
    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        match ui_catalog.get_ui_key(&current_ui_handle) {
            UiKey::MainMenu => panic!("invalid sub-ui"),
            UiKey::HostMatch => MatchLobbies::reset_host_match_state(ui_manager, &current_ui_handle),
            UiKey::JoinMatch => MatchLobbies::reset_join_match_state(ui_manager, &current_ui_handle),
            UiKey::GlobalChat => GlobalChat::reset_state(ui_manager, &current_ui_handle),
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    }

    ui_manager.set_ui_container_contents(&active_ui_handle, "center_container", &sub_ui_handle);
}
