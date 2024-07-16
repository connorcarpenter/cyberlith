use std::time::Duration;

use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::EventReader,
    schedule::NextState,
};

use game_engine::{
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    logging::info,
    render::components::RenderLayers,
    ui::UiManager,
};

use crate::{
    resources::user_manager::UserManager,
    states::AppState,
    ui::{
        events::{
            DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent,
            JoinMatchButtonClickedEvent, ResyncMainMenuUiEvent, SettingsButtonClickedEvent,
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
) {
    if AppState::Loading != current_state {
        panic!("unexpected state");
    }

    next_state.set(AppState::MainMenu);

    let layer = RenderLayers::layer(0);
    ui_manager.set_target_render_layer(layer);

    let ui_key = UiKey::MainMenu;
    let ui_handle = ui_catalog.get_ui_handle(ui_key);

    ui_catalog.set_loaded(ui_key);

    ui_manager.register_ui_event::<HostMatchButtonClickedEvent>(&ui_handle, "host_match_button");
    ui_manager.register_ui_event::<JoinMatchButtonClickedEvent>(&ui_handle, "join_match_button");
    ui_manager.register_ui_event::<GlobalChatButtonClickedEvent>(&ui_handle, "chat_button");
    ui_manager.register_ui_event::<DevlogButtonClickedEvent>(&ui_handle, "devlog_button");
    ui_manager.register_ui_event::<SettingsButtonClickedEvent>(&ui_handle, "settings_button");

    ui_manager.enable_ui(&ui_handle);

    // set sub-ui to GlobalChat at beginning
    if ui_catalog.get_is_loaded(UiKey::GlobalChat) {
        go_to_sub_ui(ui_manager, ui_catalog, UiKey::GlobalChat);
    }

    // setup user list
    user_manager.recv_main_menu_ui(ui_manager, &ui_handle);
}

pub(crate) fn handle_main_menu_interaction_events(
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

    handle_main_menu_ui_interaction_events_impl(
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

fn handle_main_menu_ui_interaction_events_impl(
    ui_manager: &mut UiManager,
    ui_catalog: &UiCatalog,
    host_match_btn_rdr: &mut EventReader<HostMatchButtonClickedEvent>,
    join_match_btn_rdr: &mut EventReader<JoinMatchButtonClickedEvent>,
    global_chat_btn_rdr: &mut EventReader<GlobalChatButtonClickedEvent>,
    devlog_btn_rdr: &mut EventReader<DevlogButtonClickedEvent>,
    settings_btn_rdr: &mut EventReader<SettingsButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // Host Match Button Click
    let mut host_match_clicked = false;
    for _ in host_match_btn_rdr.read() {
        host_match_clicked = true;
    }
    if host_match_clicked {
        info!("host match button clicked!");

        go_to_sub_ui(ui_manager, ui_catalog, UiKey::HostMatch);

        *should_rumble = true;
    }

    // Join Match Button Click
    let mut join_match_clicked = false;
    for _ in join_match_btn_rdr.read() {
        join_match_clicked = true;
    }
    if join_match_clicked {
        info!("join match button clicked!");

        go_to_sub_ui(ui_manager, ui_catalog, UiKey::JoinMatch);

        *should_rumble = true;
    }

    // Global Chat Button Click
    let mut global_chat_clicked = false;
    for _ in global_chat_btn_rdr.read() {
        global_chat_clicked = true;
    }
    if global_chat_clicked {
        info!("global chat button clicked!");

        go_to_sub_ui(ui_manager, ui_catalog, UiKey::GlobalChat);

        *should_rumble = true;
    }

    // Devlog Button Click
    let mut devlog_clicked = false;
    for _ in devlog_btn_rdr.read() {
        devlog_clicked = true;
    }
    if devlog_clicked {
        info!("devlog button clicked!");
        *should_rumble = true;
    }

    // Settings Button Click
    let mut settings_clicked = false;
    for _ in settings_btn_rdr.read() {
        settings_clicked = true;
    }
    if settings_clicked {
        info!("settings button clicked!");
        *should_rumble = true;
    }
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
