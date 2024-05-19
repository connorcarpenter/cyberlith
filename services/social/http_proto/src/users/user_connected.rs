use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserConnectedRequest {
    region_secret: String,
    user_id: UserId,
}

impl UserConnectedRequest {
    pub fn new(region_secret: &str, user_id: UserId) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            user_id,
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }

    pub fn user_id(&self) -> UserId { self.user_id }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserConnectedResponse;

// Traits
impl ApiRequest for UserConnectedRequest {
    type Response = UserConnectedResponse;

    fn name() -> &'static str { "UserConnectedRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_connected"
    }
}

impl ApiResponse for UserConnectedResponse {
    fn name() -> &'static str { "UserConnectedResponse" }
}
