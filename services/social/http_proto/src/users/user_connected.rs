use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserConnectedRequest {
    region_secret: String,
    user_id: UserId,
    user_name: String,
}

impl UserConnectedRequest {
    pub fn new(region_secret: &str, user_id: UserId, user_name: String) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            user_id,
            user_name,
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserConnectedResponse;

// Traits
impl ApiRequest for UserConnectedRequest {
    type Response = UserConnectedResponse;

    fn name() -> &'static str {
        "UserConnectedRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_connected"
    }
}

impl ApiResponse for UserConnectedResponse {
    fn name() -> &'static str {
        "UserConnectedResponse"
    }
}
