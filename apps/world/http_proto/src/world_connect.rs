use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest {
    region_secret: String,
    pub session_server_addr: String,
    pub session_server_port: u16,
    pub login_token: String,
}

impl WorldConnectRequest {
    pub fn new(
        region_secret: &str,
        session_server_addr: &str,
        session_server_port: u16,
        login_token: &str,
    ) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            session_server_addr: session_server_addr.to_string(),
            session_server_port,
            login_token: login_token.to_string(),
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct IncomingUserResponse {
    pub user_id: u64,
}

impl IncomingUserResponse {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }
}

// Traits
impl ApiRequest for WorldConnectRequest {
    type Response = IncomingUserResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "incoming_user"
    }
}

impl ApiResponse for IncomingUserResponse {}
