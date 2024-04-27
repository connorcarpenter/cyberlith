use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordResetRequest {
    pub reset_password_token: String,
    pub new_password: String,
}

impl UserPasswordResetRequest {
    pub fn new(reset_password_token: &str, new_password: &str) -> Self {
        Self {
            reset_password_token: reset_password_token.to_string(),
            new_password: new_password.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordResetResponse;

impl UserPasswordResetResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for UserPasswordResetRequest {
    type Response = UserPasswordResetResponse;

    fn name() -> &'static str {
        "UserPasswordResetRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_password_reset"
    }
}

impl ApiResponse for UserPasswordResetResponse {
    fn name() -> &'static str {
        "UserPasswordResetResponse"
    }
}
