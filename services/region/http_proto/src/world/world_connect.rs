
use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};
use social_server_types::LobbyId;
use auth_server_types::UserId;

// this is sent by the social server

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest {
    pub social_server_global_secret: String,
    pub lobby_id: LobbyId,
    // Vec<session_instance_secret, Vec<UserId>>
    pub user_ids: Vec<(String, Vec<UserId>)>,
}

impl WorldConnectRequest {
    pub fn new(social_server_global_secret: &str, lobby_id: LobbyId, user_ids: Vec<(String, Vec<UserId>)>) -> Self {
        Self {
            social_server_global_secret: social_server_global_secret.to_string(),
            lobby_id,
            user_ids,
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone, Eq, Hash)]
pub struct WorldConnectResponse {
    pub world_server_instance_secret: String,
    pub login_tokens: Vec<(UserId, String)>,
}

impl WorldConnectResponse {
    pub fn new(
        world_server_instance_secret: &str,
        login_tokens: Vec<(UserId, String)>,
    ) -> Self {
        Self {
            world_server_instance_secret: world_server_instance_secret.to_string(),
            login_tokens,
        }
    }
}

// Traits
impl ApiRequest for WorldConnectRequest {
    type Response = WorldConnectResponse;

    fn name() -> &'static str {
        "WorldConnectRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world/connect"
    }
}

impl ApiResponse for WorldConnectResponse {
    fn name() -> &'static str {
        "WorldConnectResponse"
    }
}
