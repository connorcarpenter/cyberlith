use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::{file::{WriteResult, DeleteResult, ReadDirResult, CreateDirResult, ReadResult, TaskKey}, http::ResponseKey, ui::UiHandle};

use auth_server_http_proto::{AccessTokenValidateResponse, RefreshTokenGrantResponse, UserLoginResponse, UserRegisterResponse};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataState {
    Init,
    ReadDataDir(TaskKey<ReadDirResult>),
    CreateDataDir(TaskKey<CreateDirResult>),
    DataDirExists,
    CheckForAccessToken(TaskKey<ReadResult>),
    ValidateAccessToken(ResponseKey<AccessTokenValidateResponse>),
    DeleteLocalAccessToken(TaskKey<DeleteResult>),
    FinishedAccessTokenValidation,
    CheckForRefreshToken(TaskKey<ReadResult>),
    RefreshTokenGrantAccess(ResponseKey<RefreshTokenGrantResponse>),
    StoreNewAccessToken(TaskKey<WriteResult>),
    DeleteLocalRefreshToken(TaskKey<DeleteResult>),
    CantCreateDataDir,
    Done
}

impl DataState {
    pub fn has_data_dir(&self) -> bool {
        match self {
            Self::Init | Self::ReadDataDir(_) | Self::CreateDataDir(_) | Self::CantCreateDataDir => false,
            _ => true,
        }
    }
}

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,

    pub data_state: DataState,

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
