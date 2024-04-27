use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterConfirmRequest {
    pub register_token: String,
}

impl UserRegisterConfirmRequest {
    pub fn new(register_token: &str) -> Self {
        Self {
            register_token: register_token.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterConfirmResponse {
    pub access_token: String,
}

impl UserRegisterConfirmResponse {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for UserRegisterConfirmRequest {
    type Response = UserRegisterConfirmResponse;

    fn name() -> &'static str { "UserRegisterConfirmRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_register_confirm"
    }
}

impl ApiResponse for UserRegisterConfirmResponse {
    fn name() -> &'static str { "UserRegisterConfirmResponse" }
}
