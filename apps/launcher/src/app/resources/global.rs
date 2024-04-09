use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::http::ResponseKey;

use gateway_http_proto::{UserRegisterConfirmResponse, UserRegisterResponse};

#[derive(Resource)]
pub struct Global {
    pub camera_3d: Entity,
    pub user_register_response_key_opt: Option<ResponseKey<UserRegisterResponse>>,
    pub user_register_confirm_response_key_opt: Option<ResponseKey<UserRegisterConfirmResponse>>
}

impl Default for Global {
    fn default() -> Self {
        Self {
            camera_3d: Entity::PLACEHOLDER,
            user_register_response_key_opt: None,
            user_register_confirm_response_key_opt: None,
        }
    }
}
