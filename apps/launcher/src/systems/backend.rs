use bevy_ecs::{event::EventWriter, system::ResMut};

use game_engine::{
    config::{GATEWAY_PORT, PUBLIC_IP_ADDR},
    http::HttpClient,
    kernel::AppExitAction,
    logging::{info, warn},
    ui::UiManager,
};

use gateway_http_proto::{UserLoginRequest, UserRegisterRequest};

use crate::{resources::Global, ui::go_to_ui};

pub(crate) fn backend_send_login_request(
    global: &mut Global,
    http_client: &mut HttpClient,
    ui_manager: &mut UiManager,
    user_handle: &str,
    password: &str,
) {
    if global.user_login_response_key_opt.is_some() {
        warn!("already sending login request...");
        return;
    }

    // user login request send
    let request = UserLoginRequest::new(user_handle, password);
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    global.user_login_response_key_opt = Some(key);

    info!(
        "sending login request... (userhandle: {}, password: {}",
        user_handle, password
    );

    // enable spinner
    let login_ui_handle = global.ui_login_handle.unwrap();
    ui_manager.set_node_visible(&login_ui_handle, "spinner", true);
}

pub(crate) fn backend_send_register_request(
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
    let register_ui_handle = global.ui_register_handle.unwrap();
    ui_manager.set_node_visible(&register_ui_handle, "spinner", true);
}

pub(crate) fn backend_step(
    mut global: ResMut<Global>,
    mut http_client: ResMut<HttpClient>,
    mut ui_manager: ResMut<UiManager>,
    mut app_exit_action_writer: EventWriter<AppExitAction>,
) {
    user_login_response_process(&mut global, &mut http_client, &mut ui_manager, &mut app_exit_action_writer);
    user_register_response_process(&mut global, &mut http_client, &mut ui_manager);
}

fn user_login_response_process(
    global: &mut Global,
    http_client: &mut HttpClient,
    ui_manager: &mut UiManager,
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

        let login_ui_handle = global.ui_login_handle.unwrap();
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

                ui_manager.set_text(&login_ui_handle, "error_output_text", "Invalid credentials. Please try again.");
            }
        }

        clear_spinners_if_needed(global, ui_manager);
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

        let register_ui_handle = global.ui_register_handle.unwrap();
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserRegisterResponse - 200 OK)");
            }
            Err(err) => {
                info!(
                    "client <- gateway: (UserRegisterResponse - ERROR! {})",
                    err.to_string()
                );

                ui_manager.set_text(&register_ui_handle, "error_output_text", "Oops! Something went wrong on our end. Please try again later.");
            }
        }

        clear_spinners_if_needed(global, ui_manager);

        go_to_ui(ui_manager, global, &global.ui_register_finish_handle.unwrap());
    }
}

fn clear_spinners_if_needed(global: &Global, ui_manager: &mut UiManager) {
    if global.user_login_response_key_opt.is_none() & global.user_register_response_key_opt.is_none() {
        ui_manager.set_node_visible(&global.ui_register_handle.unwrap(), "spinner", false);
        ui_manager.set_node_visible(&global.ui_login_handle.unwrap(), "spinner", false);
    }
}
