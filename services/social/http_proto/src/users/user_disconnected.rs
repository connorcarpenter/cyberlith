use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserDisconnectedRequest {
    session_instance_secret: String,
    user_id: UserId,
}

impl UserDisconnectedRequest {
    pub fn new(session_instance_secret: &str, user_id: UserId) -> Self {
        Self {
            session_instance_secret: session_instance_secret.to_string(),
            user_id,
        }
    }

    pub fn session_instance_secret(&self) -> &str {
        &self.session_instance_secret
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserDisconnectedResponse;

// Traits
impl ApiRequest for UserDisconnectedRequest {
    type Response = UserDisconnectedResponse;

    fn name() -> &'static str {
        "UserDisconnectedRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_disconnected"
    }
}

impl ApiResponse for UserDisconnectedResponse {
    fn name() -> &'static str {
        "UserDisconnectedResponse"
    }
}
