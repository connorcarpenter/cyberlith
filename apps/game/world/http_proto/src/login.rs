use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct LoginRequest {
    pub region_secret: String,
    pub login_token: String,
}

impl LoginRequest {
    pub fn new(region_secret: &str, login_token: &str) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            login_token: login_token.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct LoginResponse;

// Traits
impl ApiRequest for LoginRequest {
    type Response = LoginResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "login"
    }
}

impl ApiResponse for LoginResponse {}
