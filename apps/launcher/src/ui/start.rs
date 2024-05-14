
use bevy_ecs::event::EventReader;

use game_engine::{
    logging::info,
    ui::UiManager,
};

use crate::{ui::go_to_ui, resources::{Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent}};

pub(crate) fn handle_events(
    ui_manager: &mut UiManager,
    global: &Global,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
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
        go_to_ui(ui_manager, global, &global.ui_login_handle.unwrap());
        *should_rumble = true;
    }

    // Register Button Click
    let mut register_clicked = false;
    for _ in register_btn_rdr.read() {
        register_clicked = true;
    }
    if register_clicked {
        info!("register button clicked!");
        go_to_ui(ui_manager, global, &global.ui_register_handle.unwrap());
        *should_rumble = true;
    }

    // drain others
    for _ in submit_btn_rdr.read() {
        // ignore
    }
}