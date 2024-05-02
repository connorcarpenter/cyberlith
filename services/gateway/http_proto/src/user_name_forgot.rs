use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserNameForgotRequest {
    pub email: String,
}

impl UserNameForgotRequest {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserNameForgotResponse;

impl UserNameForgotResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for UserNameForgotRequest {
    type Response = UserNameForgotResponse;

    fn name() -> &'static str { "UserNameForgotRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "api/user_name_forgot"
    }
}

impl ApiResponse for UserNameForgotResponse {
    fn name() -> &'static str { "UserNameForgotResponse" }
}
