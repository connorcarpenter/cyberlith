use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest {
    pub session_server_instance_secret: String,
    pub user_id: UserId,
}

impl WorldConnectRequest {
    pub fn new(session_server_instance_secret: &str, user_id: UserId) -> Self {
        Self {
            session_server_instance_secret: session_server_instance_secret.to_string(),
            user_id,
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone, Eq, Hash)]
pub struct WorldConnectResponse {
    pub world_server_instance_secret: String,
    pub world_server_user_id: UserId,
    pub login_token: String,
}

impl WorldConnectResponse {
    pub fn new(
        world_server_instance_secret: &str,
        world_server_user_id: UserId,
        token: &str,
    ) -> Self {
        Self {
            world_server_instance_secret: world_server_instance_secret.to_string(),
            world_server_user_id,
            login_token: token.to_string(),
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
