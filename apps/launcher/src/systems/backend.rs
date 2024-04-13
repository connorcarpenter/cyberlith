use bevy_ecs::system::ResMut;
use logging::{info, warn};

use game_engine::{ui::UiManager, http::HttpClient, config::{GATEWAY_PORT, PUBLIC_IP_ADDR}};

use gateway_http_proto::{UserLoginRequest, UserRegisterRequest};

use crate::resources::Global;

pub(crate) fn backend_send_login_request(
    global: &mut Global,
    ui_manager: &UiManager,
    http_client: &mut HttpClient,
) {
    let login_ui_handle = global.ui_login_handle.unwrap();
    let username = ui_manager.get_textbox_text(&login_ui_handle, "username_textbox").unwrap_or("".to_string());
    let password = ui_manager.get_textbox_text(&login_ui_handle, "password_textbox").unwrap_or("".to_string());

    if global.user_login_response_key_opt.is_some() {
        warn!("already sending login request...");
        return;
    }

    // user login request send
    let request = UserLoginRequest::new(&username, &password);
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    global.user_login_response_key_opt = Some(key);
    info!("sending login request... (username: {}, password: {}", username, password);
}

pub(crate) fn backend_send_register_request(
    global: &mut Global,
    ui_manager: &UiManager,
    http_client: &mut HttpClient,
) {
    let register_ui_handle = global.ui_register_handle.unwrap();
    let username = ui_manager.get_textbox_text(&register_ui_handle, "username_textbox").unwrap_or("".to_string());
    let email = ui_manager.get_textbox_text(&register_ui_handle, "email_textbox").unwrap_or("".to_string());
    let password = ui_manager.get_textbox_text(&register_ui_handle, "password_textbox").unwrap_or("".to_string());

    if global.user_register_response_key_opt.is_some() {
        warn!("already sending register request...");
        return;
    }

    // user register request send
    let request = UserRegisterRequest::new(&username, &email, &password);
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    global.user_register_response_key_opt = Some(key);
    info!("sending register request... (username: {}, email: {}, password: {}", username, email, password);
}

pub(crate) fn backend_process_responses(
    mut global: ResMut<Global>,
    mut http_client: ResMut<HttpClient>,
) {
    user_login_response_process(&mut global, &mut http_client);
    user_register_response_process(&mut global, &mut http_client);
}

fn user_login_response_process(
    global: &mut Global,
    http_client: &mut HttpClient,
) {
    if global.user_login_response_key_opt.is_some() {
        let Some(key) = &global.user_login_response_key_opt else {
            return;
        };
        let Some(result) = http_client.recv(key) else {
            return;
        };
        global.user_login_response_key_opt = None;
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserLoginResponse - 200 OK)");

            }
            Err(err) => {
                info!(
                    "client <- gateway: (UserLoginResponse - ERROR! {})",
                    err.to_string()
                );
            }
        }
    }
}

fn user_register_response_process(
    global: &mut Global,
    http_client: &mut HttpClient,
) {
    if global.user_register_response_key_opt.is_some() {
        let Some(key) = &global.user_register_response_key_opt else {
            return;
        };
        let Some(result) = http_client.recv(key) else {
            return;
        };
        global.user_register_response_key_opt = None;
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserRegisterResponse - 200 OK)");
            }
            Err(err) => {
                info!(
                    "client <- gateway: (UserRegisterResponse - ERROR! {})",
                    err.to_string()
                );
            }
        }
    }
}