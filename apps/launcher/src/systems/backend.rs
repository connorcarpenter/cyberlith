use bevy_ecs::{event::EventWriter, system::ResMut};

use game_engine::{
    config::{GATEWAY_PORT, PUBLIC_IP_ADDR, SUBDOMAIN_API, SUBDOMAIN_WWW},
    http::HttpClient,
    kernel::AppExitAction,
    logging::{info, warn},
    ui::UiManager,
};

use auth_server_http_proto::{UserLoginRequest, UserRegisterRequest};
use game_engine::file::FileSystemManager;

use crate::resources::{DataState, Global};

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
    let url = if SUBDOMAIN_WWW.is_empty() {
        PUBLIC_IP_ADDR.to_string()
    } else {
        format!("{}.{}", SUBDOMAIN_WWW, PUBLIC_IP_ADDR)
    };
    let key = http_client.send(&url, GATEWAY_PORT, request);
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
    let url = if SUBDOMAIN_API.is_empty() {
        PUBLIC_IP_ADDR.to_string()
    } else {
        format!("{}.{}", SUBDOMAIN_API, PUBLIC_IP_ADDR)
    };
    let key = http_client.send(&url, GATEWAY_PORT, request);
    global.user_register_response_key_opt = Some(key);
    info!(
        "sending register request... (username: {}, email: {}, password: {}",
        username, email, password
    );
}

pub(crate) fn backend_step(
    mut global: ResMut<Global>,
    mut file_system_manager: ResMut<FileSystemManager>,
    mut http_client: ResMut<HttpClient>,
    mut app_exit_action_writer: EventWriter<AppExitAction>,
) {
    data_processing(&mut global, &mut file_system_manager);
    user_login_response_process(&mut global, &mut http_client, &mut file_system_manager, &mut app_exit_action_writer);
    user_register_response_process(&mut global, &mut http_client);
}

fn data_processing(
    global: &mut Global,
    fs_manager: &mut FileSystemManager,
) {
    match global.data_state.clone() {
        DataState::Init => {
            global.data_state = DataState::ReadDataDir(fs_manager.read_dir("data"));
        }
        DataState::ReadDataDir(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {

                match result {
                    Ok(_response) => {
                        global.data_state = DataState::DataDirExists;
                    }
                    Err(err) => {
                        warn!(
                            "error reading data folder: {}",
                            err.to_string()
                        );

                        let create_data_dir_key = fs_manager.create_dir("data");
                        global.data_state = DataState::CreateDataDir(create_data_dir_key)
                    }
                }
            }
        }
        DataState::CreateDataDir(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(_response) => {
                        global.data_state = DataState::DataDirExists;
                    }
                    Err(err) => {
                        warn!(
                            "error creating data folder: {}",
                            err.to_string()
                        );
                        global.data_state = DataState::Error;
                    }
                }
            }
        }
        DataState::DataDirExists => {
            let read_access_token_key = fs_manager.read("data/access_token");
            global.data_state = DataState::CheckForAccessToken(read_access_token_key)
        }
        DataState::CheckForAccessToken(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(response) => {
                        let access_token = String::from_utf8(response.bytes).unwrap();
                        global.cached_access_token = Some(access_token.clone());
                        info!("found access_token: {}", access_token);
                    }
                    Err(err) => {
                        warn!(
                            "error reading access_token: {}",
                            err.to_string()
                        );
                    }
                }
                let read_refresh_token_key = fs_manager.read("data/refresh_token");
                global.data_state = DataState::CheckForRefreshToken(read_refresh_token_key)
            }
        }
        DataState::CheckForRefreshToken(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(response) => {
                        let refresh_token = String::from_utf8(response.bytes).unwrap();
                        global.cached_refresh_token = Some(refresh_token.clone());
                        info!("found refresh_token: {}", refresh_token);
                    }
                    Err(err) => {
                        warn!(
                            "error reading refresh_token: {}",
                            err.to_string()
                        );
                    }
                }
                global.data_state = DataState::Done;
            }
        }
        DataState::Done => {}
        DataState::Error => {}
    }
}

fn user_login_response_process(
    global: &mut Global,
    http_client: &mut HttpClient,
    fs_manager: &mut FileSystemManager,
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
            Ok(response) => {
                info!("client <- gateway: (UserLoginResponse - 200 OK)");

                if global.data_state.has_data_dir() {
                    let access_token_write_key = fs_manager.write("data/access_token", response.access_token.as_bytes());
                    info!("write access token: {}", response.access_token);
                    global.store_access_token_key_opt = Some(access_token_write_key);

                    let refresh_token_write_key = fs_manager.write("data/refresh_token", response.refresh_token.as_bytes());
                    global.store_refresh_token_key_opt = Some(refresh_token_write_key);
                } else {
                    warn!("data folder does not exist, cannot store tokens locally");
                }
            }
            Err(err) => {
                warn!(
                    "client <- gateway: (UserLoginResponse - ERROR! {})",
                    err.to_string()
                );
            }
        }
    }
    if global.store_access_token_key_opt.is_some() {
        let Some(key) = &global.store_access_token_key_opt else {
            return;
        };
        let Some(result) = fs_manager.get_result(key) else {
            return;
        };
        global.store_access_token_key_opt = None;
        match result {
            Ok(_response) => {
                info!("stored access_token");
                if global.store_refresh_token_key_opt.is_none() {
                    redirect_to_game_app(app_exit_action_writer);
                } else {
                    info!("waiting for refresh_token to be stored");
                }
            }
            Err(err) => {
                warn!(
                    "error storing access_token: {}",
                    err.to_string()
                );
            }
        }
    }
    if global.store_refresh_token_key_opt.is_some() {
        let Some(key) = &global.store_refresh_token_key_opt else {
            return;
        };
        let Some(result) = fs_manager.get_result(key) else {
            return;
        };
        global.store_refresh_token_key_opt = None;
        match result {
            Ok(_response) => {
                info!("stored refresh_token");
                if global.store_access_token_key_opt.is_none() {
                    redirect_to_game_app(app_exit_action_writer);
                } else {
                    info!("waiting for access_token to be stored");
                }
            }
            Err(err) => {
                warn!(
                    "error storing refresh_token: {}",
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

fn user_register_response_process(global: &mut Global, http_client: &mut HttpClient) {
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
