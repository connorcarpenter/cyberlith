use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest {
    pub session_server_instance_secret: String,
}

impl WorldConnectRequest {
    pub fn new(session_server_instance_secret: &str) -> Self {
        Self {
            session_server_instance_secret: session_server_instance_secret.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone, Eq, Hash)]
pub struct WorldConnectResponse {
    pub world_server_instance_secret: String,
    pub world_server_user_id: u64,
    pub world_server_public_webrtc_url: String,
    pub login_token: String,
}

impl WorldConnectResponse {
    pub fn new(
        world_server_instance_secret: &str,
        world_server_user_id: u64,
        world_server_public_webrtc_url: &str,
        token: &str,
    ) -> Self {
        Self {
            world_server_instance_secret: world_server_instance_secret.to_string(),
            world_server_user_id,
            world_server_public_webrtc_url: world_server_public_webrtc_url.to_string(),
            login_token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for WorldConnectRequest {
    type Response = WorldConnectResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world/connect"
    }
}

impl ApiResponse for WorldConnectResponse {}