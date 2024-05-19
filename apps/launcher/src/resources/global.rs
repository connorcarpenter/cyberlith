use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::{http::ResponseKey, ui::UiHandle};

use gateway_http_proto::{
    ResetPasswordToken, UserLoginResponse, UserNameForgotResponse, UserPasswordForgotResponse,
    UserPasswordResetResponse, UserRegisterResponse,
};

use crate::ui::UiKey;

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,

    pub user_login_response_key_opt: Option<ResponseKey<UserLoginResponse>>,
    pub user_register_response_key_opt: Option<ResponseKey<UserRegisterResponse>>,
    pub user_name_forgot_response_key_opt: Option<ResponseKey<UserNameForgotResponse>>,
    pub user_password_forgot_response_key_opt: Option<ResponseKey<UserPasswordForgotResponse>>,
    pub user_password_reset_response_key_opt: Option<ResponseKey<UserPasswordResetResponse>>,

    ui_key_to_handle: HashMap<UiKey, UiHandle>,
    ui_handle_to_key: HashMap<UiHandle, UiKey>,

    pub reset_password_token: Option<ResetPasswordToken>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            camera_3d: Entity::PLACEHOLDER,

            user_login_response_key_opt: None,
            user_register_response_key_opt: None,
            user_name_forgot_response_key_opt: None,
            user_password_forgot_response_key_opt: None,
            user_password_reset_response_key_opt: None,

            ui_key_to_handle: HashMap::new(),
            ui_handle_to_key: HashMap::new(),

            reset_password_token: None,
        }
    }
}

impl Global {
    pub fn insert_ui(&mut self, key: UiKey, handle: UiHandle) {
        self.ui_key_to_handle.insert(key, handle);
        self.ui_handle_to_key.insert(handle, key);
    }

    pub fn get_ui_handle(&self, key: UiKey) -> &UiHandle {
        self.ui_key_to_handle.get(&key).unwrap()
    }

    pub fn get_ui_key(&self, handle: &UiHandle) -> &UiKey {
        self.ui_handle_to_key.get(handle).unwrap()
    }
}
