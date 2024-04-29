use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::{file::{WriteResult, ReadDirResult, CreateDirResult, ReadResult, TaskKey}, http::ResponseKey, ui::UiHandle};

use auth_server_http_proto::{UserLoginResponse, UserRegisterResponse};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataState {
    Init,
    ReadDataDir(TaskKey<ReadDirResult>),
    CreateDataDir(TaskKey<CreateDirResult>),
    DataDirExists,
    CheckForAccessToken(TaskKey<ReadResult>),
    CheckForRefreshToken(TaskKey<ReadResult>),
    Error,
    Done
}

impl DataState {
    pub fn has_data_dir(&self) -> bool {
        match self {
            Self::DataDirExists | Self::CheckForAccessToken(_) | Self::CheckForRefreshToken(_) | Self::Done => true,
            _ => false,
        }
    }
}

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,

    // pub user_register_response_key_opt: Option<ResponseKey<UserRegisterResponse>>,
    // pub user_register_confirm_response_key_opt: Option<ResponseKey<UserRegisterConfirmResponse>>,
    // pub user_password_forgot_response_key_opt: Option<ResponseKey<UserPasswordForgotResponse>>,
    // pub user_password_reset_response_key_opt: Option<ResponseKey<UserPasswordResetResponse>>,

    pub data_state: DataState,
    pub cached_access_token: Option<String>,
    pub cached_refresh_token: Option<String>,

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

            data_state: DataState::Init,
            cached_access_token: None,
            cached_refresh_token: None,

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
