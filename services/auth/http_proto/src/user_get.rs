use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

use auth_server_types::{UserId, UserRole};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserGetRequest {
    pub user_id: UserId,
}

impl UserGetRequest {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserGetResponse {
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

impl UserGetResponse {
    pub fn new(name: String, email: String, role: UserRole) -> Self {
        Self { name, email, role }
    }
}

// Traits
impl ApiRequest for UserGetRequest {
    type Response = UserGetResponse;

    fn name() -> &'static str {
        "UserGetRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_get"
    }
}

impl ApiResponse for UserGetResponse {
    fn name() -> &'static str {
        "UserGetResponse"
    }
}
