use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordForgotRequest {
    pub email: String,
}

impl UserPasswordForgotRequest {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordForgotResponse;

impl UserPasswordForgotResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for UserPasswordForgotRequest {
    type Response = UserPasswordForgotResponse;

    fn name() -> &'static str { "UserPasswordForgotRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "api/user_password_forgot"
    }
}

impl ApiResponse for UserPasswordForgotResponse {
    fn name() -> &'static str { "UserPasswordForgotResponse" }
}
