
use bevy_ecs::event::EventReader;

use game_engine::{
    logging::info,
    ui::UiManager,
};

use crate::{ui::go_to_ui, resources::{Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent}};

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Register Finish Ui

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("home button clicked!");
        // go to start ui
        go_to_ui(ui_manager, global, &global.ui_start_handle.unwrap());
        *should_rumble = true;
    }

    // drain others
    for _ in login_btn_rdr.read() {
        // ignore
    }
    for _ in register_btn_rdr.read() {
        // ignore
    }
}