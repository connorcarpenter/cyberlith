
use bevy_ecs::event::EventReader;

use game_engine::{
    http::HttpClient,
    logging::info,
    ui::UiManager,
};

use crate::{
    resources::{
        Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
    },
    systems::backend::backend_send_login_request,
};

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    http_client: &mut HttpClient,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Login Ui

    // Register Button Click
    let mut register_clicked = false;
    for _ in register_btn_rdr.read() {
        register_clicked = true;
    }
    if register_clicked {
        info!("register button clicked!");
        ui_manager.enable_ui(&global.ui_register_handle.unwrap());
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");

        // get data from textboxes
        let login_ui_handle = global.ui_login_handle.unwrap();
        let user_handle = ui_manager
            .get_textbox_text(&login_ui_handle, "username_textbox")
            .unwrap_or("".to_string());
        let password = ui_manager
            .get_textbox_text(&login_ui_handle, "password_textbox")
            .unwrap_or("".to_string());

        // validate
        // check that every field is not empty
        if user_handle.is_empty() {
            ui_manager.set_text(&login_ui_handle, "error_output_text", "Please enter your username.");
            return;
        }
        if password.is_empty() {
            ui_manager.set_text(&login_ui_handle, "error_output_text", "Please enter your password.");
            return;
        }

        // check that every field matches the necessary minimum length
        if user_handle.len() < ui_manager.get_textbox_validator(&login_ui_handle, "username_textbox").unwrap().min_length() {
            ui_manager.set_text(&login_ui_handle, "error_output_text", "Username is invalid.");
            return;
        }
        if password.len() < ui_manager.get_textbox_validator(&login_ui_handle, "password_textbox").unwrap().min_length() {
            ui_manager.set_text(&login_ui_handle, "error_output_text", "Password is invalid.");
            return;
        }

        // send backend request
        backend_send_login_request(global, http_client, &user_handle, &password);

        *should_rumble = true;
    }

    // drain others
    for _ in login_btn_rdr.read() {
        // ignore
    }
}
