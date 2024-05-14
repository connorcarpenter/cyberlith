
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
    systems::backend::backend_send_register_request,
    ui::go_to_ui
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
    // in Register Ui

    // Login Button Click
    let mut login_clicked = false;
    for _ in login_btn_rdr.read() {
        login_clicked = true;
    }
    if login_clicked {
        info!("login button clicked!");

        // TODO: validate!

        go_to_ui(ui_manager, global, &global.ui_login_handle.unwrap());
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");

        // rumble
        *should_rumble = true;

        // get data from textboxes
        let register_ui_handle = global.ui_register_handle.unwrap();
        let username = ui_manager
            .get_textbox_text(&register_ui_handle, "username_textbox")
            .unwrap_or("".to_string());
        let email = ui_manager
            .get_textbox_text(&register_ui_handle, "email_textbox")
            .unwrap_or("".to_string());
        let password = ui_manager
            .get_textbox_text(&register_ui_handle, "password_textbox")
            .unwrap_or("".to_string());
        let confirm_password = ui_manager
            .get_textbox_text(&register_ui_handle, "confirm_password_textbox")
            .unwrap_or("".to_string());

        // validate

        // check that the passwords match
        if !password.eq(&confirm_password) {
            ui_manager.set_text(&register_ui_handle, "error_output_text", "Passwords do not match. Please try again.");
            return;
        }

        // check that every field is not empty
        if username.is_empty() {
            ui_manager.set_text(&register_ui_handle, "error_output_text", "Please enter your username.");
            return;
        }
        if email.is_empty() {
            ui_manager.set_text(&register_ui_handle, "error_output_text", "Please enter your email address.");
            return;
        }
        if password.is_empty() {
            ui_manager.set_text(&register_ui_handle, "error_output_text", "Please enter your password.");
            return;
        }
        if confirm_password.is_empty() {
            ui_manager.set_text(&register_ui_handle, "error_output_text", "Please confirm your password.");
            return;
        }

        // check that every field matches the necessary minimum length
        {
            let min_length = ui_manager.get_textbox_validator(&register_ui_handle, "username_textbox").unwrap().min_length();
            if username.len() < min_length {
                let error_text = format!(
                    "Username must be at least {} characters long.",
                    min_length,
                );
                ui_manager.set_text(&register_ui_handle, "error_output_text", &error_text);
                return;
            }
        }
        {
            let min_length = ui_manager.get_textbox_validator(&register_ui_handle, "email_textbox").unwrap().min_length();
            if email.len() < min_length {
                let error_text = format!(
                    "Email must be at least {} characters long.",
                    min_length,
                );
                ui_manager.set_text(&register_ui_handle, "error_output_text", &error_text);
                return;
            }
        }
        {
            let min_length = ui_manager.get_textbox_validator(&register_ui_handle, "password_textbox").unwrap().min_length();
            if password.len() < min_length {
                let error_text = format!(
                    "Password must be at least {} characters long.",
                    min_length,
                );
                ui_manager.set_text(&register_ui_handle, "error_output_text", &error_text);
                return;
            }
        }

        // send backend request
        backend_send_register_request(global, http_client, ui_manager, &username, &email, &password);
    }

    // drain others
    for _ in register_btn_rdr.read() {
        // ignore
    }
}