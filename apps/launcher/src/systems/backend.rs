use bevy_ecs::{event::EventWriter, system::ResMut};

use game_engine::{
    config::GATEWAY_PORT,
    http::HttpClient,
    kernel::AppExitAction,
    logging::{info, warn},
    ui::UiManager,
    file::FileSystemManager,
};

use auth_server_http_proto::{AccessToken, AccessTokenValidateRequest, RefreshToken, RefreshTokenGrantRequest, UserLoginRequest, UserRegisterRequest};

use crate::{utils::{get_api_url, get_www_url}, resources::{DataState, Global}};

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
    let key = http_client.send(&get_api_url(), GATEWAY_PORT, request);
    global.user_register_response_key_opt = Some(key);
    info!(
        "sending register request... (username: {}, email: {}, password: {}",
        username, email, password
    );
}

pub(crate) fn backend_step(
    mut global: ResMut<Global>,
    mut fs_manager: ResMut<FileSystemManager>,
    mut http_client: ResMut<HttpClient>,
    mut ui_manager: ResMut<UiManager>,
    mut app_exit_action_writer: EventWriter<AppExitAction>,
) {
    data_processing(&mut global, &mut fs_manager, &mut http_client, &mut app_exit_action_writer);
    user_login_response_process(&mut global, &mut http_client, &mut fs_manager, &mut app_exit_action_writer);
    user_register_response_process(&mut global, &mut http_client, &mut ui_manager);
}

fn data_processing(
    global: &mut Global,
    fs_manager: &mut FileSystemManager,
    http_client: &mut HttpClient,
    app_exit_action_writer: &mut EventWriter<AppExitAction>,
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
                        global.data_state = DataState::CantCreateDataDir;
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
                        let access_token = AccessToken::from_str(&String::from_utf8(response.bytes).unwrap()).unwrap();
                        info!("found access_token in fs: {:?}", access_token);

                        // validate access token via http
                        let request = AccessTokenValidateRequest::new(access_token);
                        let response_key = http_client.send(&get_www_url(), GATEWAY_PORT, request);
                        info!("sending access token validate request: {:?}", access_token);
                        global.data_state = DataState::ValidateAccessToken(response_key);
                    }
                    Err(err) => {
                        warn!(
                            "error reading access_token: {}",
                            err.to_string()
                        );
                        global.data_state = DataState::FinishedAccessTokenValidation;
                    }
                }
            }
        }
        DataState::ValidateAccessToken(response_key) => {
            if let Some(result) = http_client.recv(&response_key) {
                match result {
                    Ok(_response) => {
                        info!("access token is valid, redirecting to game app...");
                        redirect_to_game_app(app_exit_action_writer);
                    }
                    Err(err) => {
                        warn!(
                            "access token is invalid: {}",
                            err.to_string()
                        );
                        // DELETE ACCESS TOKEN from FS
                        let delete_access_token_key = fs_manager.delete("data/access_token");
                        global.data_state = DataState::DeleteLocalAccessToken(delete_access_token_key);
                    }
                }
            }
        }
        DataState::DeleteLocalAccessToken(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(_response) => {
                        info!("deleted access_token from fs");
                    }
                    Err(err) => {
                        warn!(
                            "error deleting access_token: {}",
                            err.to_string()
                        );
                    }
                }
                global.data_state = DataState::FinishedAccessTokenValidation;
            }
        }
        DataState::FinishedAccessTokenValidation => {
            let read_refresh_token_key = fs_manager.read("data/refresh_token");
            global.data_state = DataState::CheckForRefreshToken(read_refresh_token_key)
        }
        DataState::CheckForRefreshToken(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(response) => {
                        let refresh_token = RefreshToken::from_str(String::from_utf8(response.bytes).unwrap().as_str()).unwrap();
                        info!("found refresh_token in fs: {:?}", refresh_token);

                        // use refresh token to get new access token via http
                        let request = RefreshTokenGrantRequest::new(refresh_token);
                        let response_key = http_client.send(&get_www_url(), GATEWAY_PORT, request);
                        info!("sending refresh token grant request: {:?}", refresh_token);
                        global.data_state = DataState::RefreshTokenGrantAccess(response_key);
                    }
                    Err(err) => {
                        warn!(
                            "error reading refresh_token: {}",
                            err.to_string()
                        );
                        global.data_state = DataState::Done;
                    }
                }
            }
        }
        DataState::RefreshTokenGrantAccess(response_key) => {
            if let Some(result) = http_client.recv(&response_key) {
                match result {
                    Ok(response) => {

                        let access_token = response.access_token.to_string();
                        info!("refresh token granted new access token... {:?}", access_token);

                        // store new access token in fs
                        let store_access_token_key = fs_manager.write("data/access_token", access_token.as_bytes());
                        global.data_state = DataState::StoreNewAccessToken(store_access_token_key);
                    }
                    Err(err) => {
                        warn!(
                            "access token is invalid: {}",
                            err.to_string()
                        );
                        // DELETE REFRESH TOKEN from FS
                        let delete_refresh_token_key = fs_manager.delete("data/refresh_token");
                        global.data_state = DataState::DeleteLocalRefreshToken(delete_refresh_token_key);
                    }
                }
            }
        }
        DataState::StoreNewAccessToken(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(_response) => {
                        info!("stored new access_token, redirecting to app");
                        redirect_to_game_app(app_exit_action_writer);
                    }
                    Err(err) => {
                        warn!(
                            "error storing new access_token: {}",
                            err.to_string()
                        );
                        global.data_state = DataState::Done;
                    }
                }
            }
        }
        DataState::DeleteLocalRefreshToken(task_key) => {
            if let Some(result) = fs_manager.get_result(&task_key) {
                match result {
                    Ok(_response) => {
                        info!("deleted refresh_token from fs");
                    }
                    Err(err) => {
                        warn!(
                            "error deleting refresh_token: {}",
                            err.to_string()
                        );
                    }
                }
                global.data_state = DataState::Done;
            }
        }
        DataState::Done => {}
        DataState::CantCreateDataDir => {}
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
                    let access_token_write_key = fs_manager.write("data/access_token", response.access_token.to_string().as_bytes());
                    info!("write access token: {:?}", response.access_token);
                    global.store_access_token_key_opt = Some(access_token_write_key);

                    let refresh_token_write_key = fs_manager.write("data/refresh_token", response.refresh_token.to_string().as_bytes());
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
