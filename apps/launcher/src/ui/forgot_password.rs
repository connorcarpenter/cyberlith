
use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    asset::{AssetId, embedded_asset_event, EmbeddedAssetEvent},
    http::HttpClient,
    ui::{UiManager, UiHandle},
    logging::{warn, info},
    config::{GATEWAY_PORT, PUBLIC_IP_ADDR},
};
use gateway_http_proto::UserPasswordForgotRequest;

use crate::{ui::{UiKey, clear_spinners_if_needed, go_to_ui}, resources::{
    Global, TextboxClickedEvent, SubmitButtonClickedEvent, BackButtonClickedEvent,
}};

pub(crate) fn setup(
    global: &mut Global,
    ui_manager: &mut UiManager,
    embedded_asset_events: &mut EventWriter<EmbeddedAssetEvent>,
    ui_key: UiKey,
) {
    embedded_asset_events.send(embedded_asset_event!("../embedded/m25ed3"));

    let ui_handle = UiHandle::new(AssetId::from_str("m25ed3").unwrap());
    global.insert_ui(ui_key, ui_handle);
    ui_manager.register_ui_event::<BackButtonClickedEvent>(&ui_handle, "back_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "email_textbox");
}

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    http_client: &mut HttpClient,
    back_btn_rdr: &mut EventReader<BackButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    textbox_click_rdr: &mut EventReader<TextboxClickedEvent>,
    should_rumble: &mut bool,
) {
    // Back Button Click
    let mut back_btn_clicked = false;
    for _ in back_btn_rdr.read() {
        back_btn_clicked = true;
    }
    if back_btn_clicked {
        info!("back button clicked!");
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::Login));
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_btn_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_btn_clicked = true;
    }
    if submit_btn_clicked {
        info!("submit button clicked!");

        let ui_handle = global.get_ui_handle(UiKey::ForgotUsername);

        // rumble
        *should_rumble = true;

        // clear error output
        ui_manager.set_text(&ui_handle, "error_output_text", "");

        // get data from textboxes
        let email = ui_manager
            .get_textbox_text(&ui_handle, "email_textbox")
            .unwrap_or("".to_string());

        // validate

        // check that every field is not empty
        if email.is_empty() {
            ui_manager.set_text(&ui_handle, "error_output_text", "Please enter your email address.");
            return;
        }

        // check that every field matches the necessary minimum length
        {
            let min_length = ui_manager.get_textbox_validator(&ui_handle, "email_textbox").unwrap().min_length();
            if email.len() < min_length {
                let error_text = format!(
                    "Email must be at least {} characters long.",
                    min_length,
                );
                ui_manager.set_text(&ui_handle, "error_output_text", &error_text);
                return;
            }
        }

        // send backend request
        backend_send_request(global, http_client, ui_manager, &email);
    }

    // Textbox Click
    let mut textbox_clicked = false;
    for _ in textbox_click_rdr.read() {
        textbox_clicked = true;
    }
    if textbox_clicked {
        info!("textbox clicked!");

        let ui_handle = global.get_ui_handle(UiKey::ForgotPassword);
        ui_manager.set_text(&ui_handle, "error_output_text", "");
    }
}

pub fn reset_state(
    ui_manager: &mut UiManager,
    ui_handle: &UiHandle
) {
    // clear textboxes
    ui_manager.set_text(&ui_handle, "email_textbox", "");

    // clear error output
    ui_manager.set_text(&ui_handle, "error_output_text", "");

    // clear spinner
    ui_manager.set_node_visible(&ui_handle, "spinner", false);
}

fn backend_send_request(
    global: &mut Global,
    http_client: &mut HttpClient,
    ui_manager: &mut UiManager,
    email: &str,
) {

    if global.user_password_forgot_response_key_opt.is_some() {
        warn!("already sending password forgot request...");
        return;
    }

    // password forgot request send
    let request = UserPasswordForgotRequest::new(&email);
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    global.user_password_forgot_response_key_opt = Some(key);
    info!("sending password forgot request... (email: {})", email);

    // enable spinner
    let ui_handle = global.get_ui_handle(UiKey::ForgotUsername);
    ui_manager.set_node_visible(&ui_handle, "spinner", true);
}

pub(crate) fn process_requests(
    global: &mut Global,
    http_client: &mut HttpClient,
    ui_manager: &mut UiManager,
) {
    if global.user_password_forgot_response_key_opt.is_some() {
        let key = global.user_password_forgot_response_key_opt.as_ref().unwrap();
        let Some(result) = http_client.recv(key) else {
            return;
        };
        global.user_password_forgot_response_key_opt = None;

        let ui_handle = global.get_ui_handle(UiKey::ForgotPassword);
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserPasswordForgotResponse - 200 OK)");
            }
            Err(err) => {
                info!(
                    "client <- gateway: (UserPasswordForgotResponse - ERROR! {})",
                    err.to_string()
                );

                ui_manager.set_text(&ui_handle, "error_output_text", "Oops! Something went wrong on our end. Please try again later.");
            }
        }

        clear_spinners_if_needed(global, ui_manager);

        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::ForgotPasswordFinish));
    }
}