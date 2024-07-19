use naia_serde::SerdeInternal as Serde;

use auth_server_types::UserId;
use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::LobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialWorldConnectRequest {
    social_secret: String,
    world_instance_secret: String,
    lobby_id: LobbyId,
    login_tokens: Vec<(UserId, String)>
}

impl SocialWorldConnectRequest {
    pub fn new(social_secret: &str, world_instance_secret: &str, lobby_id: LobbyId, login_tokens: Vec<(UserId, String)>) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            world_instance_secret: world_instance_secret.to_string(),
            lobby_id,
            login_tokens
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn world_instance_secret(&self) -> &str {
        &self.world_instance_secret
    }

    pub fn lobby_id(&self) -> LobbyId {
        self.lobby_id
    }

    pub fn login_tokens(&self) -> &Vec<(UserId, String)> {
        &self.login_tokens
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialWorldConnectResponse;

// Traits
impl ApiRequest for SocialWorldConnectRequest {
    type Response = SocialWorldConnectResponse;

    fn name() -> &'static str {
        "SocialWorldConnectRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_world_connect"
    }
}

impl ApiResponse for SocialWorldConnectResponse {
    fn name() -> &'static str {
        "SocialWorldConnectResponse"
    }
}
