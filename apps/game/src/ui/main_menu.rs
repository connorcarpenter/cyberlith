use std::time::Duration;

use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::{EventReader, EventWriter},
    schedule::NextState,
    system::Query,
};

use game_engine::{
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    logging::{info},
    render::components::RenderLayers,
    ui::UiManager,
    session::{SessionClient, components::Lobby},
};

use crate::{
    resources::{user_manager::UserManager, lobby_manager::LobbyManager, match_manager::MatchManager},
    states::AppState,
    ui::{
        events::{
            DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent,
            JoinMatchButtonClickedEvent, ResyncMainMenuUiEvent, SettingsButtonClickedEvent, GoToSubUiEvent, CurrentLobbyButtonClickedEvent, LeaveLobbyButtonClickedEvent, StartMatchButtonClickedEvent
        },
        go_to_sub_ui, UiCatalog, UiKey,
    },
};

pub(crate) fn on_main_menu_ui_load(
    current_state: AppState,
    next_state: &mut NextState<AppState>,
    ui_catalog: &mut UiCatalog,
    ui_manager: &mut UiManager,
    user_manager: &mut UserManager,
    sub_ui_event_writer: &mut EventWriter<GoToSubUiEvent>,
) {
    if AppState::Loading != current_state {
        panic!("unexpected state");
    }

    next_state.set(AppState::MainMenu);

    let layer = RenderLayers::layer(0);
    ui_manager.set_target_render_layer(layer);

    let main_menu_ui_key = UiKey::MainMenu;
    let main_menu_ui_handle = ui_catalog.get_ui_handle(main_menu_ui_key);

    ui_catalog.set_loaded(main_menu_ui_key);

    ui_manager.register_ui_event::<HostMatchButtonClickedEvent>(&main_menu_ui_handle, "host_match_button");
    ui_manager.register_ui_event::<JoinMatchButtonClickedEvent>(&main_menu_ui_handle, "join_match_button");
    ui_manager.register_ui_event::<GlobalChatButtonClickedEvent>(&main_menu_ui_handle, "chat_button");
    ui_manager.register_ui_event::<DevlogButtonClickedEvent>(&main_menu_ui_handle, "devlog_button");
    ui_manager.register_ui_event::<SettingsButtonClickedEvent>(&main_menu_ui_handle, "settings_button");
    ui_manager.register_ui_event::<CurrentLobbyButtonClickedEvent>(&main_menu_ui_handle, "current_lobby_button");
    ui_manager.register_ui_event::<StartMatchButtonClickedEvent>(&main_menu_ui_handle, "start_button");
    ui_manager.register_ui_event::<LeaveLobbyButtonClickedEvent>(&main_menu_ui_handle, "leave_button");

    ui_manager.enable_ui(&main_menu_ui_handle);

    // set sub-ui to GlobalChat at beginning
    if ui_catalog.get_is_loaded(UiKey::MessageList) {
        go_to_sub_ui(sub_ui_event_writer, UiKey::MessageList);
    }

    // setup user list
    user_manager.recv_main_menu_ui(ui_manager, &main_menu_ui_handle);
}

pub(crate) fn handle_main_menu_interaction_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    ui_manager: Res<UiManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut sub_ui_event_writer: EventWriter<GoToSubUiEvent>,
    mut host_match_btn_rdr: EventReader<HostMatchButtonClickedEvent>,
    mut join_match_btn_rdr: EventReader<JoinMatchButtonClickedEvent>,
    mut global_chat_btn_rdr: EventReader<GlobalChatButtonClickedEvent>,
    mut devlog_btn_rdr: EventReader<DevlogButtonClickedEvent>,
    mut settings_btn_rdr: EventReader<SettingsButtonClickedEvent>,
    mut current_lobby_btn_rdr: EventReader<CurrentLobbyButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    // Host Match Button Click
    {
        let mut host_match_clicked = false;
        for _ in host_match_btn_rdr.read() {
            host_match_clicked = true;
        }
        if host_match_clicked {
            info!("host match button clicked!");

            go_to_sub_ui(&mut sub_ui_event_writer, UiKey::HostMatch);

            should_rumble = true;
        }
    }

    // Join Match Button Click
    {
        let mut join_match_clicked = false;
        for _ in join_match_btn_rdr.read() {
            join_match_clicked = true;
        }
        if join_match_clicked {
            info!("join match button clicked!");

            go_to_sub_ui(&mut sub_ui_event_writer, UiKey::JoinMatch);

            should_rumble = true;
        }
    }

    // Global Chat Button Click
    {
        let mut global_chat_clicked = false;
        for _ in global_chat_btn_rdr.read() {
            global_chat_clicked = true;
        }
        if global_chat_clicked {
            info!("global chat button clicked!");

            go_to_sub_ui(&mut sub_ui_event_writer, UiKey::MessageList);

            should_rumble = true;
        }
    }

    // Devlog Button Click
    {
        let mut devlog_clicked = false;
        for _ in devlog_btn_rdr.read() {
            devlog_clicked = true;
        }
        if devlog_clicked {
            info!("devlog button clicked!");
            should_rumble = true;
        }
    }

    // Settings Button Click
    {
        let mut settings_clicked = false;
        for _ in settings_btn_rdr.read() {
            settings_clicked = true;
        }
        if settings_clicked {
            info!("settings button clicked!");
            should_rumble = true;
        }
    }

    // Current Lobby Button Click
    {
        let mut current_lobby_clicked = false;
        for _ in current_lobby_btn_rdr.read() {
            current_lobby_clicked = true;
        }
        if current_lobby_clicked {
            info!("current lobby button clicked!");
            should_rumble = true;
        }
    }

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

pub(crate) fn handle_start_match_events(
    ui_manager: Res<UiManager>,
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut session_client: SessionClient,
    mut lobby_manager: ResMut<LobbyManager>,
    mut match_manager: ResMut<MatchManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut resync_main_menu_ui_events: EventWriter<ResyncMainMenuUiEvent>,
    mut start_match_btn_rdr: EventReader<StartMatchButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    lobby_manager.handle_start_match_events(
        &mut session_client,
        &mut match_manager,
        &mut resync_main_menu_ui_events,
        &mut start_match_btn_rdr,
        &mut should_rumble,
    );

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

pub(crate) fn handle_leave_lobby_events(
    ui_manager: Res<UiManager>,
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut session_client: SessionClient,
    mut lobby_manager: ResMut<LobbyManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut resync_main_menu_ui_events: EventWriter<ResyncMainMenuUiEvent>,
    mut resync_chat_message_ui_events: EventWriter<ResyncMessageListUiEvent>,
    mut resync_user_ui_events: EventWriter<ResyncUserListUiEvent>,
    mut leave_lobby_btn_rdr: EventReader<LeaveLobbyButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    lobby_manager.handle_leave_lobby_events(
        &mut session_client,
        &mut resync_main_menu_ui_events,
        &mut resync_chat_message_ui_events,
        &mut resync_user_ui_events,
        &mut leave_lobby_btn_rdr,
        &mut should_rumble,
    );

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

pub(crate) fn handle_resync_main_menu_ui_events(
    session_client: SessionClient,
    mut ui_manager: ResMut<UiManager>,
    ui_catalog: Res<UiCatalog>,
    user_manager: Res<UserManager>,
    lobby_manager: Res<LobbyManager>,
    match_manager: Res<MatchManager>,
    lobby_q: Query<&Lobby>,
    mut resync_main_menu_ui_events: EventReader<ResyncMainMenuUiEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }
    let Some(current_sub_ui_handle) = ui_manager.get_ui_container_contents(&active_ui_handle, "center_container") else {
        return;
    };
    let current_sub_ui_key = ui_catalog.get_ui_key(&current_sub_ui_handle);

    // check if we need to resync
    let mut resync = false;
    for _ in resync_main_menu_ui_events.read() {
        resync = true;
    }
    if !resync {
        return;
    }

    // we must resync

    if match_manager.in_match() {
        // in a match
        ui_manager.set_button_enabled(&active_ui_handle, "host_match_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "join_match_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "chat_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "current_lobby_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "start_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "leave_button", false);
        // TODO: disable chat bar

        return;
    }

    let current_lobby_id = lobby_manager.get_current_lobby();

    if let Some(current_lobby_id) = current_lobby_id {
        // in a lobby

        // disable "host_match" "join_match" "chat" buttons
        ui_manager.set_button_enabled(&active_ui_handle, "host_match_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "join_match_button", false);
        ui_manager.set_button_enabled(&active_ui_handle, "chat_button", false);

        // get current lobby
        let current_lobby_entity = lobby_manager.get_lobby_entity(&current_lobby_id).unwrap();
        let current_lobby = lobby_q.get(current_lobby_entity).unwrap();

        // make left side "lobby" button visible
        ui_manager.set_node_visible(&active_ui_handle, "current_lobby_button", true);
        ui_manager.set_text(&active_ui_handle, "current_lobby_button_text", &current_lobby.name);
        ui_manager.set_text(&active_ui_handle, "center_title_text", &current_lobby.name);

        // make right side "leave lobby" button visible
        ui_manager.set_node_visible(&active_ui_handle, "leave_button", true);
        ui_manager.set_button_enabled(&active_ui_handle, "leave_button", true);

        // make right side "start match" button visible (if host)
        let self_is_owner_of_lobby: bool = {
            let lobby_owner_user_entity = current_lobby.owner_user_entity.get(&session_client).unwrap();

            let self_user_entity = user_manager.get_self_user_entity().unwrap();

            lobby_owner_user_entity == self_user_entity
        };

        if self_is_owner_of_lobby {
            ui_manager.set_node_visible(&active_ui_handle, "start_button", true);
            ui_manager.set_button_enabled(&active_ui_handle, "start_button", true);
        }
    } else {
        // not in a lobby

        // enable "host_match" "join_match" "chat" buttons
        ui_manager.set_button_enabled(&active_ui_handle, "host_match_button", true);
        ui_manager.set_button_enabled(&active_ui_handle, "join_match_button", true);
        ui_manager.set_button_enabled(&active_ui_handle, "chat_button", true);

        // make left side "lobby" button invisible
        ui_manager.set_node_visible(&active_ui_handle, "current_lobby_button", false);

        // make right side "leave lobby" button invisible
        ui_manager.set_node_visible(&active_ui_handle, "leave_button", false);

        // make right side "start match" button invisible
        ui_manager.set_node_visible(&active_ui_handle, "start_button", false);

        // set center title text appropriately
        let center_title_text = match current_sub_ui_key {
            UiKey::HostMatch => "Host Match",
            UiKey::JoinMatch => "Join Match",
            UiKey::MessageList => "Chat",
            _ => {
                panic!("unexpected sub ui");
            }
        };
        ui_manager.set_text(&active_ui_handle, "center_title_text", center_title_text);
    }
}
