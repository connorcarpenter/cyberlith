use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::{file::{WriteResult, ReadDirResult, CreateDirResult, TaskKey}, http::ResponseKey, ui::UiHandle};

use auth_server_http_proto::{UserLoginResponse, UserRegisterResponse};

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,

    // pub user_register_response_key_opt: Option<ResponseKey<UserRegisterResponse>>,
    // pub user_register_confirm_response_key_opt: Option<ResponseKey<UserRegisterConfirmResponse>>,
    // pub user_password_forgot_response_key_opt: Option<ResponseKey<UserPasswordForgotResponse>>,
    // pub user_password_reset_response_key_opt: Option<ResponseKey<UserPasswordResetResponse>>,

    pub has_data_dir: bool,
    pub read_data_dir_key_opt: Option<TaskKey<ReadDirResult>>,
    pub create_data_dir_key_opt: Option<TaskKey<CreateDirResult>>,

    pub user_login_response_key_opt: Option<ResponseKey<UserLoginResponse>>,
    pub store_access_token_key_opt: Option<TaskKey<WriteResult>>,
    pub store_refresh_token_key_opt: Option<TaskKey<WriteResult>>,
    pub user_register_response_key_opt: Option<ResponseKey<UserRegisterResponse>>,

    pub ui_start_handle: Option<UiHandle>,
    pub ui_register_handle: Option<UiHandle>,
    pub ui_login_handle: Option<UiHandle>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            camera_3d: Entity::PLACEHOLDER,

            // user_register_response_key_opt: None,
            // user_register_confirm_response_key_opt: None,
            // user_password_forgot_response_key_opt: None,
            // user_password_reset_response_key_opt: None,
            has_data_dir: false,
            read_data_dir_key_opt: None,
            create_data_dir_key_opt: None,

            user_login_response_key_opt: None,
            store_access_token_key_opt: None,
            store_refresh_token_key_opt: None,

            user_register_response_key_opt: None,

            ui_start_handle: None,
            ui_register_handle: None,
            ui_login_handle: None,
        }
    }
}
