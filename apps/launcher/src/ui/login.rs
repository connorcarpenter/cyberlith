
use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    http::HttpClient,
    logging::info,
    ui::{UiManager, UiHandle},
    asset::{AssetId, embedded_asset_event, EmbeddedAssetEvent},
};

use crate::{
    resources::{
        Global, TextboxClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent, ForgotPasswordButtonClickedEvent, ForgotUsernameButtonClickedEvent
    },
    systems::backend::backend_send_login_request, ui::{go_to_ui, UiKey}
};

pub(crate) fn setup(
    global: &mut Global,
    ui_manager: &mut UiManager,
    embedded_asset_events: &mut EventWriter<EmbeddedAssetEvent>,
    ui_key: UiKey,
) {
    embedded_asset_events.send(embedded_asset_event!("../embedded/3f5gej"));

    let ui_handle = UiHandle::new(AssetId::from_str("3f5gej").unwrap());
    global.insert_ui(ui_key, ui_handle);
    ui_manager.register_ui_event::<RegisterButtonClickedEvent>(&ui_handle, "register_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
    ui_manager.register_ui_event::<ForgotUsernameButtonClickedEvent>(&ui_handle, "forgot_username_button");
    ui_manager.register_ui_event::<ForgotPasswordButtonClickedEvent>(&ui_handle, "forgot_password_button");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "username_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "password_textbox");
}

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    http_client: &mut HttpClient,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    forgot_username_btn_rdr: &mut EventReader<ForgotUsernameButtonClickedEvent>,
    forgot_password_btn_rdr: &mut EventReader<ForgotPasswordButtonClickedEvent>,
    textbox_click_rdr: &mut EventReader<TextboxClickedEvent>,
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
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::Register));
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");

        let login_ui_handle = global.get_ui_handle(UiKey::Login);

        // clear error output
        ui_manager.set_text(&login_ui_handle, "error_output_text", "");

        // get data from textboxes
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
        backend_send_login_request(global, http_client, ui_manager, &user_handle, &password);

        *should_rumble = true;
    }

    // Forgot Username Button Click
    let mut forgot_username_btn_clicked = false;
    for _ in forgot_username_btn_rdr.read() {
        forgot_username_btn_clicked = true;
    }
    if forgot_username_btn_clicked {
        info!("forgot username button clicked!");
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::ForgotUsername));
        *should_rumble = true;
    }

    // Forgot Password Button Click
    let mut forgot_password_btn_clicked = false;
    for _ in forgot_password_btn_rdr.read() {
        forgot_password_btn_clicked = true;
    }
    if forgot_password_btn_clicked {
        info!("forgot password button clicked!");
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::ForgotPassword));
        *should_rumble = true;
    }

    // Textbox Click
    let mut textbox_clicked = false;
    for _ in textbox_click_rdr.read() {
        textbox_clicked = true;
    }
    if textbox_clicked {
        info!("textbox clicked!");

        let login_ui_handle = global.get_ui_handle(UiKey::Login);
        ui_manager.set_text(&login_ui_handle, "error_output_text", "");
    }
}

pub fn reset_state(
    ui_manager: &mut UiManager,
    ui_handle: &UiHandle
) {

    // clear textboxes
    ui_manager.set_text(&ui_handle, "username_textbox", "");
    ui_manager.set_text(&ui_handle, "password_textbox", "");
    ui_manager.set_textbox_password_eye_visible(&ui_handle, "password_textbox", false);

    // clear error output
    ui_manager.set_text(&ui_handle, "error_output_text", "");

    // clear spinner
    ui_manager.set_node_visible(&ui_handle, "spinner", false);
}