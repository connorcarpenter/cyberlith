use bevy_ecs::{schedule::NextState, event::{EventReader}};

use game_engine::{
    logging::info,
    ui::{UiHandle, UiManager},
    render::components::RenderLayers,
};

use crate::{
    states::AppState,
    ui::{UiCatalog, UiKey, events::{DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent, JoinMatchButtonClickedEvent, SettingsButtonClickedEvent}},
};
use crate::ui::go_to_sub_ui;

pub(crate) fn on_load(
    current_state: AppState,
    next_state: &mut NextState<AppState>,
    ui_catalog: &mut UiCatalog,
    ui_manager: &mut UiManager,
) {
    if AppState::Loading != current_state {
        panic!("unexpected state");
    }

    next_state.set(AppState::MainMenu);

    let layer = RenderLayers::layer(0);
    ui_manager.set_target_render_layer(layer);

    let ui_key = UiKey::MainMenu;
    let ui_handle = UiHandle::new(UiCatalog::game_main_menu_ui());

    ui_catalog.set_loaded(ui_key);

    ui_manager.register_ui_event::<HostMatchButtonClickedEvent>(&ui_handle, "host_match_button");
    ui_manager.register_ui_event::<JoinMatchButtonClickedEvent>(&ui_handle, "join_match_button");
    ui_manager.register_ui_event::<GlobalChatButtonClickedEvent>(&ui_handle, "chat_button");
    ui_manager.register_ui_event::<DevlogButtonClickedEvent>(&ui_handle, "devlog_button");
    ui_manager.register_ui_event::<SettingsButtonClickedEvent>(&ui_handle, "settings_button");

    ui_manager.enable_ui(&ui_handle);
}

pub(crate) fn handle_events(
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

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    panic!("should never leave main menu?")
}
