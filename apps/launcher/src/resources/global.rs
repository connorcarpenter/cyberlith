use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::{http::ResponseKey, ui::UiHandle};

use gateway_http_proto::{UserLoginResponse, UserRegisterResponse};

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,

    // pub user_register_response_key_opt: Option<ResponseKey<UserRegisterResponse>>,
    // pub user_register_confirm_response_key_opt: Option<ResponseKey<UserRegisterConfirmResponse>>,
    // pub user_password_forgot_response_key_opt: Option<ResponseKey<UserPasswordForgotResponse>>,
    // pub user_password_reset_response_key_opt: Option<ResponseKey<UserPasswordResetResponse>>,
    pub user_login_response_key_opt: Option<ResponseKey<UserLoginResponse>>,
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
            user_login_response_key_opt: None,
            user_register_response_key_opt: None,

            ui_start_handle: None,
            ui_register_handle: None,
            ui_login_handle: None,
        }
    }
}