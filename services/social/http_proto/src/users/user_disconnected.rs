use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserDisconnectedRequest {
    session_secret: String,
    user_id: UserId,
}

impl UserDisconnectedRequest {
    pub fn new(session_secret: &str, user_id: UserId) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            user_id,
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }

    pub fn user_id(&self) -> UserId { self.user_id }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserDisconnectedResponse;

// Traits
impl ApiRequest for UserDisconnectedRequest {
    type Response = UserDisconnectedResponse;

    fn name() -> &'static str { "UserDisconnectedRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_disconnected"
    }
}

impl ApiResponse for UserDisconnectedResponse {
    fn name() -> &'static str { "UserDisconnectedResponse" }
}
