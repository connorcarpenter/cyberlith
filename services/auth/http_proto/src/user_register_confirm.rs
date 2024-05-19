use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

use crate::{AccessToken, RefreshToken, RegisterToken};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterConfirmRequest {
    pub register_token: RegisterToken,
}

impl UserRegisterConfirmRequest {
    pub fn new(register_token: RegisterToken) -> Self {
        Self { register_token }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterConfirmResponse {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

impl UserRegisterConfirmResponse {
    pub fn new(access_token: AccessToken, refresh_token: RefreshToken) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }
}

// Traits
impl ApiRequest for UserRegisterConfirmRequest {
    type Response = UserRegisterConfirmResponse;

    fn name() -> &'static str {
        "UserRegisterConfirmRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_register_confirm"
    }
}

impl ApiResponse for UserRegisterConfirmResponse {
    fn name() -> &'static str {
        "UserRegisterConfirmResponse"
    }
}
