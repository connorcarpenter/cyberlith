use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest {
    region_secret: String,
    pub session_server_addr: String,
    pub session_server_port: u16,
    pub user_id: UserId,
    pub login_token: String,
}

impl WorldConnectRequest {
    pub fn new(
        region_secret: &str,
        session_server_addr: &str,
        session_server_port: u16,
        user_id: UserId,
        login_token: &str,
    ) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            session_server_addr: session_server_addr.to_string(),
            session_server_port,
            user_id,
            login_token: login_token.to_string(),
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct IncomingUserResponse;

impl IncomingUserResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for WorldConnectRequest {
    type Response = IncomingUserResponse;

    fn name() -> &'static str { "WorldConnectRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "incoming_user"
    }
}

impl ApiResponse for IncomingUserResponse {
    fn name() -> &'static str { "IncomingUserResponse" }
}
