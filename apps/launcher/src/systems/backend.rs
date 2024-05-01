use bevy_ecs::{event::EventWriter, system::ResMut};

use game_engine::{
    config::GATEWAY_PORT,
    http::HttpClient,
    kernel::AppExitAction,
    logging::{info, warn},
    ui::UiManager,
};

use gateway_http_proto::{UserLoginRequest, UserRegisterRequest};

use crate::{utils::get_www_url, resources::Global};

pub(crate) fn backend_send_login_request(
    global: &mut Global,
    ui_manager: &UiManager,
    http_client: &mut HttpClient,
) {
    let login_ui_handle = global.ui_login_handle.unwrap();
    let username = ui_manager
        .get_textbox_text(&login_ui_handle, "username_textbox")
        .unwrap_or("".to_string());
    let password = ui_manager
        .get_textbox_text(&login_ui_handle, "password_textbox")
        .unwrap_or("".to_string());

    if global.user_login_response_key_opt.is_some() {
        warn!("already sending login request...");
        return;
    }

    // user login request send
    let request = UserLoginRequest::new(&username, &password);
    let key = http_client.send(&get_www_url(), GATEWAY_PORT, request);
    global.user_login_response_key_opt = Some(key);
    info!(
        "sending login request... (username: {}, password: {}",
        username, password
    );
}

pub(crate) fn backend_send_register_request(
    global: &mut Global,
    ui_manager: &UiManager,
    http_client: &mut HttpClient,
) {
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

    if global.user_register_response_key_opt.is_some() {
        warn!("already sending register request...");
        return;
    }

    // user register request send
    let request = UserRegisterRequest::new(&username, &email, &password);
    let key = http_client.send(&get_www_url(), GATEWAY_PORT, request);
    global.user_register_response_key_opt = Some(key);
    info!(
        "sending register request... (username: {}, email: {}, password: {}",
        username, email, password
    );
}

pub(crate) fn backend_step(
    mut global: ResMut<Global>,
    mut http_client: ResMut<HttpClient>,
    mut ui_manager: ResMut<UiManager>,
    mut app_exit_action_writer: EventWriter<AppExitAction>,
) {
    user_login_response_process(&mut global, &mut http_client, &mut app_exit_action_writer);
    user_register_response_process(&mut global, &mut http_client, &mut ui_manager);
}

fn user_login_response_process(
    global: &mut Global,
    http_client: &mut HttpClient,
    app_exit_action_writer: &mut EventWriter<AppExitAction>,
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

                redirect_to_game_app(app_exit_action_writer);
            }
            Err(err) => {
                warn!(
                    "client <- gateway: (UserLoginResponse - ERROR! {})",
                    err.to_string()
                );
            }
        }
    }
}

fn redirect_to_game_app(app_exit_action_writer: &mut EventWriter<AppExitAction>) {
    info!("redirecting to game app");
    app_exit_action_writer.send(AppExitAction::go_to("game"));
}

fn user_register_response_process(
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
        ui_manager.enable_ui(&global.ui_register_finish_handle.unwrap());
    }
}
