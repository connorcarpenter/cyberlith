use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserIsOnlineRequest {
    user_id: UserId,
}

impl UserIsOnlineRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserIsOnlineResponse {
    online: bool,
}

impl UserIsOnlineResponse {
    pub fn offline() -> Self {
        Self { online: false }
    }

    pub fn online() -> Self {
        Self { online: true }
    }

    pub fn is_online(&self) -> bool {
        self.online
    }
}

// Traits
impl ApiRequest for UserIsOnlineRequest {
    type Response = UserIsOnlineResponse;

    fn name() -> &'static str {
        "UserIsOnlineRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_is_online"
    }
}

impl ApiResponse for UserIsOnlineResponse {
    fn name() -> &'static str {
        "UserIsOnlineResponse"
    }
}
