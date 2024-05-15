
use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    logging::info,
    ui::{UiManager, UiHandle},
    asset::{AssetId, embedded_asset_event, EmbeddedAssetEvent},
};

use crate::{ui::{go_to_ui, UiKey}, resources::{Global, LoginButtonClickedEvent, RegisterButtonClickedEvent}};

pub(crate) fn setup(
    global: &mut Global,
    ui_manager: &mut UiManager,
    embedded_asset_events: &mut EventWriter<EmbeddedAssetEvent>,
    ui_key: UiKey
) {
    embedded_asset_events.send(embedded_asset_event!("../embedded/tpp7za"));

    let ui_handle = UiHandle::new(AssetId::from_str("tpp7za").unwrap());
    global.insert_ui(ui_key, ui_handle);
    ui_manager.register_ui_event::<LoginButtonClickedEvent>(&ui_handle, "login_button");
    ui_manager.register_ui_event::<RegisterButtonClickedEvent>(&ui_handle, "register_button");
}

pub(crate) fn handle_events(
    ui_manager: &mut UiManager,
    global: &Global,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Start Ui

    // Login Button Click
    let mut login_clicked = false;
    for _ in login_btn_rdr.read() {
        login_clicked = true;
    }
    if login_clicked {
        info!("login button clicked!");
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::Login));
        *should_rumble = true;
    }

    // Register Button Click
    let mut register_clicked = false;
    for _ in register_btn_rdr.read() {
        register_clicked = true;
    }
    if register_clicked {
        info!("register button clicked!");
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::Register));
        *should_rumble = true;
    }
}

pub fn reset_state(
    ui_manager: &mut UiManager,
    ui_handle: &UiHandle
) {

}