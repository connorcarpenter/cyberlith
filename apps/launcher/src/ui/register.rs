use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    asset::{embedded_asset_event, AssetId, EmbeddedAssetEvent},
    config::{GATEWAY_PORT, PUBLIC_IP_ADDR},
    http::HttpClient,
    logging::{info, warn},
    ui::{UiHandle, UiManager},
};
use gateway_http_proto::UserRegisterRequest;

use crate::ui::clear_spinners_if_needed;
use crate::{
    resources::{Global, LoginButtonClickedEvent, SubmitButtonClickedEvent, TextboxClickedEvent},
    ui::{go_to_ui, UiKey},
};

pub(crate) fn setup(
    global: &mut Global,
    ui_manager: &mut UiManager,
    embedded_asset_events: &mut EventWriter<EmbeddedAssetEvent>,
    ui_key: UiKey,
) {
    embedded_asset_events.send(embedded_asset_event!("../embedded/rckneg"));

    let ui_handle = UiHandle::new(AssetId::from_str("rckneg").unwrap());
    global.insert_ui(ui_key, ui_handle);
    ui_manager.register_ui_event::<LoginButtonClickedEvent>(&ui_handle, "login_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "username_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "email_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "password_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "confirm_password_textbox");
}

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    http_client: &mut HttpClient,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    textbox_click_rdr: &mut EventReader<TextboxClickedEvent>,
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

        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::Login));
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");

        let register_ui_handle = global.get_ui_handle(UiKey::Register);

        // rumble
        *should_rumble = true;

        // clear error output
        ui_manager.set_text(&register_ui_handle, "error_output_text", "");

        // get data from textboxes
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
            ui_manager.set_text(
                &register_ui_handle,
                "error_output_text",
                "Passwords do not match. Please try again.",
            );
            return;
        }

        // check that every field is not empty
        if username.is_empty() {
            ui_manager.set_text(
                &register_ui_handle,
                "error_output_text",
                "Please enter your username.",
            );
            return;
        }
        if email.is_empty() {
            ui_manager.set_text(
                &register_ui_handle,
                "error_output_text",
                "Please enter your email address.",
            );
            return;
        }
        if password.is_empty() {
            ui_manager.set_text(
                &register_ui_handle,
                "error_output_text",
                "Please enter your password.",
            );
            return;
        }
        if confirm_password.is_empty() {
            ui_manager.set_text(
                &register_ui_handle,
                "error_output_text",
                "Please confirm your password.",
            );
            return;
        }

        // check that every field matches the necessary minimum length
        {
            let min_length = ui_manager
                .get_textbox_validator(&register_ui_handle, "username_textbox")
                .unwrap()
                .min_length();
            if username.len() < min_length {
                let error_text =
                    format!("Username must be at least {} characters long.", min_length,);
                ui_manager.set_text(&register_ui_handle, "error_output_text", &error_text);
                return;
            }
        }
        {
            let min_length = ui_manager
                .get_textbox_validator(&register_ui_handle, "email_textbox")
                .unwrap()
                .min_length();
            if email.len() < min_length {
                let error_text = format!("Email must be at least {} characters long.", min_length,);
                ui_manager.set_text(&register_ui_handle, "error_output_text", &error_text);
                return;
            }
        }
        {
            let min_length = ui_manager
                .get_textbox_validator(&register_ui_handle, "password_textbox")
                .unwrap()
                .min_length();
            if password.len() < min_length {
                let error_text =
                    format!("Password must be at least {} characters long.", min_length,);
                ui_manager.set_text(&register_ui_handle, "error_output_text", &error_text);
                return;
            }
        }

        // send backend request
        backend_send_request(
            global,
            http_client,
            ui_manager,
            &username,
            &email,
            &password,
        );
    }

    // Textbox Click
    let mut textbox_clicked = false;
    for _ in textbox_click_rdr.read() {
        textbox_clicked = true;
    }
    if textbox_clicked {
        info!("textbox clicked!");

        let register_ui_handle = global.get_ui_handle(UiKey::Register);
        ui_manager.set_text(&register_ui_handle, "error_output_text", "");
    }
}

pub fn reset_state(ui_manager: &mut UiManager, ui_handle: &UiHandle) {
    // clear textboxes
    ui_manager.set_text(&ui_handle, "username_textbox", "");
    ui_manager.set_text(&ui_handle, "email_textbox", "");
    ui_manager.set_text(&ui_handle, "password_textbox", "");
    ui_manager.set_textbox_password_eye_visible(&ui_handle, "password_textbox", false);
    ui_manager.set_text(&ui_handle, "confirm_password_textbox", "");
    ui_manager.set_textbox_password_eye_visible(&ui_handle, "confirm_password_textbox", false);

    // clear error output
    ui_manager.set_text(&ui_handle, "error_output_text", "");

    // clear spinner
    ui_manager.set_node_visible(&ui_handle, "spinner", false);
}

fn backend_send_request(
    global: &mut Global,
    http_client: &mut HttpClient,
    ui_manager: &mut UiManager,
    username: &str,
    email: &str,
    password: &str,
) {
    if global.user_register_response_key_opt.is_some() {
        warn!("already sending register request...");
        return;
    }

    // user register request send
    let request = UserRegisterRequest::new(&username, &email, &password);
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    global.user_register_response_key_opt = Some(key);
    info!(
        "sending register request... (username: {}, email: {}, password: {}",
        username, email, password
    );

    // enable spinner
    let register_ui_handle = global.get_ui_handle(UiKey::Register);
    ui_manager.set_node_visible(&register_ui_handle, "spinner", true);
}

pub(crate) fn process_requests(
    global: &mut Global,
    http_client: &mut HttpClient,
    ui_manager: &mut UiManager,
) {
    if global.user_register_response_key_opt.is_some() {
        let key = global.user_register_response_key_opt.as_ref().unwrap();
        let Some(result) = http_client.recv(key) else {
            return;
        };
        global.user_register_response_key_opt = None;

        let register_ui_handle = global.get_ui_handle(UiKey::Register);
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserRegisterResponse - 200 OK)");
            }
            Err(err) => {
                info!(
                    "client <- gateway: (UserRegisterResponse - ERROR! {})",
                    err.to_string()
                );

                ui_manager.set_text(
                    &register_ui_handle,
                    "error_output_text",
                    "Oops! Something went wrong on our end. Please try again later.",
                );
            }
        }

        clear_spinners_if_needed(global, ui_manager);

        go_to_ui(
            ui_manager,
            global,
            &global.get_ui_handle(UiKey::RegisterFinish),
        );
    }
}
